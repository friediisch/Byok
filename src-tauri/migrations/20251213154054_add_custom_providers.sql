-- Add custom provider support
-- New columns: base_url, api_scheme, is_custom

-- Add base_url column (optional, for custom providers)
ALTER TABLE providers ADD COLUMN base_url TEXT DEFAULT NULL;

-- Add api_scheme column (openai, anthropic, mistral, etc.)
-- This determines which API format to use when making requests
ALTER TABLE providers ADD COLUMN api_scheme TEXT DEFAULT NULL;

-- Add is_custom column to distinguish user-added providers
ALTER TABLE providers ADD COLUMN is_custom BOOLEAN DEFAULT FALSE;

-- Set api_scheme for existing built-in providers
UPDATE providers SET api_scheme = 'openai' WHERE provider_name = 'openai';
UPDATE providers SET api_scheme = 'anthropic' WHERE provider_name = 'anthropic';
UPDATE providers SET api_scheme = 'mistral' WHERE provider_name = 'mistralai';
UPDATE providers SET api_scheme = 'groq' WHERE provider_name = 'groqcloud';

