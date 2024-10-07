-- Up migration

CREATE TABLE IF NOT EXISTS providers
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    api_key TEXT DEFAULT '',
    display_name TEXT NOT NULL,
    api_key_valid BOOLEAN DEFAULT FALSE
);
INSERT INTO providers (provider_name, display_name) VALUES ('openai', 'OpenAI');
INSERT INTO providers (provider_name, display_name) VALUES ('anthropic', 'Anthropic');
INSERT INTO providers (provider_name, display_name) VALUES ('mistralai', 'Mistral AI');
INSERT INTO providers (provider_name, display_name) VALUES ('groqcloud', 'Groq Cloud');

CREATE TABLE IF NOT EXISTS models
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_display_name TEXT NOT NULL, 
    show BOOLEAN DEFAULT 1,
    max_tokens INTEGER DEFAULT 4096,
    context_window INTEGER DEFAULT 1024
);

CREATE TABLE IF NOT EXISTS chats
(
    id TEXT NOT NULL PRIMARY KEY,
    model TEXT,
    api_key_id TEXT,
    display_name TEXT NOT NULL,
    archived TEXT,
    creation_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_updated DATETIME 
);
CREATE INDEX idx_last_updated ON chats(last_updated DESC);

CREATE TABLE IF NOT EXISTS messages
(
    id TEXT NOT NULL PRIMARY KEY,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    chat_id TEXT NOT NULL,
    model_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS message_blocks
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER,
    type_ TEXT NOT NULL,
    language TEXT,
    raw_content TEXT NOT NULL,
    rendered_content TEXT NOT NULL,
    copied INTEGER DEFAULT FALSE
);