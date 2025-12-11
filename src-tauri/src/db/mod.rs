//! Database module - handles all database operations
//!
//! This module is organized into submodules by domain:
//! - `init` - Database initialization and default models
//! - `chats` - Chat CRUD operations
//! - `messages` - Message CRUD operations
//! - `models` - Model CRUD operations
//! - `providers_db` - Provider/API key operations

// Make submodules public so Tauri command macros can access generated symbols
pub mod chats;
pub mod init;
pub mod messages;
pub mod models;
pub mod providers_db;

// Re-export initialization
pub use init::{init, DEFAULT_MODELS};

// Re-export chat operations
pub use chats::{archive_chat, delete_chat, get_chat_display_name, get_chats, insert_chat_display_name, rename_chat};

// Re-export message operations
pub use messages::{get_messages, insert_message, insert_message_blocks, load_chat};

// Re-export model operations
pub use models::{add_model, delete_model, get_all_models, get_models, update_model};

// Re-export provider operations
pub use providers_db::{get_api_key, load_providers, read_api_keys_from_env, set_api_key};
