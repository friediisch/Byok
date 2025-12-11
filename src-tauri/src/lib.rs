#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
// Allow unexpected_cfgs from objc crate macros
#![allow(unexpected_cfgs)]

use dotenv::dotenv;
use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem, SubmenuBuilder};
use tauri::{command, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

mod data;
mod db;
mod errors;
mod llm_providers;
mod providers;
mod settings;
mod types;
mod utils;

pub use errors::{AppError, AppResult, ConfigError, DatabaseError, DbResult, ProviderError, ProviderResult};

use crate::data::{AppPaths, AppSettings, ArcData, Data, DbPool, MainWindow, PathsState};
use std::sync::Arc;

fn error_popup_main_thread(msg: impl AsRef<str>) {
	let msg = msg.as_ref().to_string();
	let builder = rfd::MessageDialog::new()
		.set_title("Error")
		.set_description(&msg)
		.set_buttons(rfd::MessageButtons::Ok)
		.set_level(rfd::MessageLevel::Error);
	builder.show();
}

#[macro_export]
macro_rules! throw {
	($($arg:tt)*) => {{
		return Err(format!($($arg)*))
	}};
}

#[command]
#[specta::specta]
fn error_popup(msg: String, win: tauri::WebviewWindow) {
	eprintln!("Error: {}", msg);
	win.dialog()
		.message(msg)
		.title("Error")
		.kind(tauri_plugin_dialog::MessageDialogKind::Error)
		.show(|_button_press| {});
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
	dotenv().ok();
	env_logger::init();

	#[cfg(debug_assertions)]
	{
		let specta_builder = tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
			error_popup,
			providers::get_message,
			db::chats::get_chats,
			db::messages::load_chat,
			db::providers_db::load_providers,
			db::providers_db::set_api_key,
			db::models::get_models,
			db::models::get_all_models,
			db::models::add_model,
			db::models::update_model,
			db::models::delete_model,
			db::providers_db::read_api_keys_from_env,
			db::chats::rename_chat,
			db::chats::archive_chat,
			db::chats::delete_chat,
			settings::get_settings,
			settings::apply_and_save_settings
		]);
		specta_builder
			.export(specta_typescript::Typescript::default(), "../bindings.ts")
			.expect("Failed to export typescript bindings");
		println!("Generated TS types");
	}

	let ctx = tauri::generate_context!();

	// macOS "App Nap" periodically pauses our app when it's in the background.
	// We need to prevent that so our intervals are not interrupted.
	#[cfg(target_os = "macos")]
	macos_app_nap::prevent();

	// Initialize app paths and database before the Tauri builder
	let app_paths = AppPaths::from_identifier(&ctx.config().identifier);
	let pool = match db::init(&app_paths).await {
		Ok(pool) => pool,
		Err(e) => {
			error_popup_main_thread(&e);
			panic!("{}", e);
		}
	};

	let app = tauri::Builder::default()
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![
			error_popup,
			providers::get_message,
			db::chats::get_chats,
			db::messages::load_chat,
			db::providers_db::load_providers,
			db::providers_db::set_api_key,
			db::models::get_models,
			db::models::get_all_models,
			db::models::add_model,
			db::models::update_model,
			db::models::delete_model,
			db::providers_db::read_api_keys_from_env,
			db::chats::rename_chat,
			db::chats::archive_chat,
			db::chats::delete_chat,
			settings::get_settings,
			settings::apply_and_save_settings
		])
		.setup(move |app| {
			// Get the window that was created from tauri.conf.json
			let win = app.get_webview_window("main").expect("main window not found");

			#[cfg(target_os = "macos")]
			#[allow(deprecated)]
			{
				use cocoa::appkit::NSWindow;
				let nsw = win.ns_window().unwrap() as cocoa::base::id;
				unsafe {
					// set window to always be dark mode
					use cocoa::appkit::NSAppearanceNameVibrantDark;
					use objc::*;
					let appearance: cocoa::base::id = msg_send![
						class!(NSAppearance),
						appearanceNamed: NSAppearanceNameVibrantDark
					];
					let () = msg_send![nsw, setAppearance: appearance];

					// set window background color
					let bg_color = cocoa::appkit::NSColor::colorWithRed_green_blue_alpha_(cocoa::base::nil, 34.0 / 255.0, 38.0 / 255.0, 45.5 / 255.0, 1.0);
					nsw.setBackgroundColor_(bg_color);
				}
			}
			let settings_path = &app_paths.settings_file.clone();
			let loaded_settings = settings::Settings::load(settings_path);

			// Register new separate states for better lock granularity
			app.manage(DbPool(pool.clone()));
			app.manage(AppSettings::new(loaded_settings.clone()));
			app.manage(PathsState(app_paths.clone()));
			app.manage(MainWindow(Arc::new(win.clone())));

			// Also register combined state for backwards compatibility during migration
			let data: Data = Data {
				db_pool: pool.clone(),
				paths: app_paths.clone(),
				window: win.clone(),
				settings: loaded_settings,
			};
			app.manage(ArcData::new(data));

			Ok(())
		})
		.menu(|app| {
			let package_name = &app.package_info().name;

			#[cfg(target_os = "macos")]
			let app_menu = SubmenuBuilder::new(app, package_name)
				.item(&PredefinedMenuItem::about(app, Some(package_name), None)?)
				.separator()
				.item(&MenuItem::with_id(app, "preferences", "Preferences...", true, Some("CmdOrCtrl+,"))?)
				.separator()
				.item(&PredefinedMenuItem::services(app, None)?)
				.separator()
				.item(&PredefinedMenuItem::hide(app, None)?)
				.item(&PredefinedMenuItem::hide_others(app, None)?)
				.item(&PredefinedMenuItem::show_all(app, None)?)
				.separator()
				.item(&PredefinedMenuItem::quit(app, None)?)
				.build()?;

			let file_menu = {
				#[allow(unused_mut)]
				let mut builder = SubmenuBuilder::new(app, "File").item(&MenuItem::with_id(app, "new_chat", "New Chat", true, Some("CmdOrCtrl+N"))?);

				#[cfg(not(target_os = "macos"))]
				{
					builder = builder
						.separator()
						.item(&MenuItem::with_id(app, "options", "Options...", true, Some("CmdOrCtrl+,"))?);
				}

				builder.separator().item(&PredefinedMenuItem::close_window(app, None)?).build()?
			};

			let edit_menu = {
				#[allow(unused_mut)]
				let mut builder = SubmenuBuilder::new(app, "Edit")
					.item(&PredefinedMenuItem::undo(app, None)?)
					.item(&PredefinedMenuItem::redo(app, None)?)
					.separator()
					.item(&PredefinedMenuItem::cut(app, None)?)
					.item(&PredefinedMenuItem::copy(app, None)?)
					.item(&PredefinedMenuItem::paste(app, None)?);

				#[cfg(not(target_os = "macos"))]
				{
					builder = builder.separator();
				}

				builder
					.item(&PredefinedMenuItem::select_all(app, None)?)
					.separator()
					.item(&MenuItem::with_id(app, "find", "Find", true, Some("CmdOrCtrl+F"))?)
					.build()?
			};

			let view_menu = SubmenuBuilder::new(app, "View").item(&PredefinedMenuItem::fullscreen(app, None)?).build()?;

			let window_menu = SubmenuBuilder::new(app, "Window")
				.item(&PredefinedMenuItem::minimize(app, None)?)
				.item(&PredefinedMenuItem::maximize(app, None)?)
				.build()?;

			let help_menu = SubmenuBuilder::new(app, "Help")
				.item(&MenuItem::with_id(app, "get_started", "Get Started", true, None::<&str>)?)
				.item(&MenuItem::with_id(app, "learn_more", "Learn More", true, None::<&str>)?)
				.build()?;

			#[cfg(target_os = "macos")]
			let menu = MenuBuilder::new(app)
				.item(&app_menu)
				.item(&file_menu)
				.item(&edit_menu)
				.item(&view_menu)
				.item(&window_menu)
				.item(&help_menu)
				.build()?;

			#[cfg(not(target_os = "macos"))]
			let menu = MenuBuilder::new(app)
				.item(&file_menu)
				.item(&edit_menu)
				.item(&view_menu)
				.item(&window_menu)
				.item(&help_menu)
				.build()?;

			Ok(menu)
		})
		.on_menu_event(|app, event| match event.id().as_ref() {
			"new_chat" => {
				let _ = app.emit("menuNewChat", ());
			}
			"preferences" | "options" => {
				let _ = app.emit("menuOpenSettings", ());
			}
			"learn_more" => {
				let url = "https://github.com/friediisch/GenHub";
				let _ = tauri_plugin_opener::open_url(url, None::<&str>);
			}
			_ => {}
		})
		.build(ctx)
		.expect("Error running tauri app");

	app.run(|_app_handle, e| match e {
		tauri::RunEvent::WindowEvent { event, .. } => match event {
			tauri::WindowEvent::CloseRequested { api: _api, .. } => {
				#[cfg(target_os = "macos")]
				#[allow(deprecated)]
				{
					// hide the application
					// manual for now (PR https://github.com/tauri-apps/tauri/pull/3689)
					_api.prevent_close();
					use objc::*;
					let cls = objc::runtime::Class::get("NSApplication").unwrap();
					let app: cocoa::base::id = unsafe { msg_send![cls, sharedApplication] };
					unsafe { msg_send![app, hide: 0] }
				}
			}
			_ => {}
		},
		_ => {}
	});
}
