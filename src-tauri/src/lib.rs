pub mod data;
pub mod db;
pub mod llm_providers;
pub mod providers;
pub mod settings;
pub mod types;
pub mod utils;

#[macro_export]
macro_rules! throw {
	($($arg:tt)*) => {{
		return Err(format!($($arg)*))
	}};
}
