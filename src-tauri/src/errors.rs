//! Application error types for better error handling and context

use std::fmt;

/// Application-level error types
#[derive(Debug)]
pub enum AppError {
	/// Database-related errors
	Database(DatabaseError),
	/// LLM provider errors
	Provider(ProviderError),
	/// Settings/configuration errors
	Config(ConfigError),
	/// Generic error with message
	Generic(String),
}

impl fmt::Display for AppError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			AppError::Database(e) => write!(f, "Database error: {}", e),
			AppError::Provider(e) => write!(f, "Provider error: {}", e),
			AppError::Config(e) => write!(f, "Configuration error: {}", e),
			AppError::Generic(msg) => write!(f, "{}", msg),
		}
	}
}

impl std::error::Error for AppError {}

impl From<DatabaseError> for AppError {
	fn from(e: DatabaseError) -> Self {
		AppError::Database(e)
	}
}

impl From<ProviderError> for AppError {
	fn from(e: ProviderError) -> Self {
		AppError::Provider(e)
	}
}

impl From<ConfigError> for AppError {
	fn from(e: ConfigError) -> Self {
		AppError::Config(e)
	}
}

impl From<String> for AppError {
	fn from(s: String) -> Self {
		AppError::Generic(s)
	}
}

impl From<&str> for AppError {
	fn from(s: &str) -> Self {
		AppError::Generic(s.to_string())
	}
}

// Convert to String for Tauri command returns
impl From<AppError> for String {
	fn from(e: AppError) -> Self {
		e.to_string()
	}
}

/// Database-specific errors
#[derive(Debug)]
pub enum DatabaseError {
	/// Failed to connect to database
	Connection(String),
	/// Failed to run migrations
	Migration(String),
	/// Query execution failed
	Query { operation: String, details: String },
	/// Record not found
	NotFound { entity: String, id: String },
	/// Duplicate record
	Duplicate { entity: String, id: String },
}

impl fmt::Display for DatabaseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			DatabaseError::Connection(msg) => write!(f, "Failed to connect to database: {}", msg),
			DatabaseError::Migration(msg) => write!(f, "Database migration failed: {}", msg),
			DatabaseError::Query { operation, details } => {
				write!(f, "Database {} failed: {}", operation, details)
			}
			DatabaseError::NotFound { entity, id } => {
				write!(f, "{} with id '{}' not found", entity, id)
			}
			DatabaseError::Duplicate { entity, id } => {
				write!(f, "{} with id '{}' already exists", entity, id)
			}
		}
	}
}

impl std::error::Error for DatabaseError {}

impl From<sqlx::Error> for DatabaseError {
	fn from(e: sqlx::Error) -> Self {
		DatabaseError::Query {
			operation: "query".to_string(),
			details: e.to_string(),
		}
	}
}

/// LLM Provider-specific errors
#[derive(Debug)]
pub enum ProviderError {
	/// Provider not supported
	Unsupported(String),
	/// Invalid API key
	InvalidApiKey { provider: String, reason: String },
	/// Failed to send message
	MessageFailed { provider: String, details: String },
	/// Rate limited
	RateLimited { provider: String },
	/// Provider configuration error
	Configuration { provider: String, details: String },
}

impl fmt::Display for ProviderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ProviderError::Unsupported(provider) => {
				write!(f, "Unsupported provider: {}", provider)
			}
			ProviderError::InvalidApiKey { provider, reason } => {
				write!(f, "Invalid API key for {}: {}", provider, reason)
			}
			ProviderError::MessageFailed { provider, details } => {
				write!(f, "Failed to send message to {}: {}", provider, details)
			}
			ProviderError::RateLimited { provider } => {
				write!(f, "Rate limited by {}", provider)
			}
			ProviderError::Configuration { provider, details } => {
				write!(f, "Configuration error for {}: {}", provider, details)
			}
		}
	}
}

impl std::error::Error for ProviderError {}

/// Configuration/settings errors
#[derive(Debug)]
pub enum ConfigError {
	/// Failed to load settings file
	LoadFailed { path: String, reason: String },
	/// Failed to save settings file
	SaveFailed { path: String, reason: String },
	/// Invalid setting value
	InvalidValue { setting: String, reason: String },
}

impl fmt::Display for ConfigError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ConfigError::LoadFailed { path, reason } => {
				write!(f, "Failed to load settings from '{}': {}", path, reason)
			}
			ConfigError::SaveFailed { path, reason } => {
				write!(f, "Failed to save settings to '{}': {}", path, reason)
			}
			ConfigError::InvalidValue { setting, reason } => {
				write!(f, "Invalid value for '{}': {}", setting, reason)
			}
		}
	}
}

impl std::error::Error for ConfigError {}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

/// Result type alias for database operations
pub type DbResult<T> = Result<T, DatabaseError>;

/// Result type alias for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;
