# Byok

### Todos:

##### High Priority

- Use `sqlx prepare` to compile-time validate types in queries
- Handle errors within the LLM providers
  - Include a note saying that LLM providers can be down sometimes
  - Handle such cases gracefully
- Check out Kalosm for local inference
  - https://docs.rs/kalosm/latest/kalosm/
  - https://github.com/EricLBuehler/mistral.rs
- Add way to add new model to the database
  - add way to hide models
- Add proper logging with levels and stuff (when applicable)
  - https://v2.tauri.app/plugin/logging/
- Add API-support
  - Google: Add support for Google as soon as it is available in Germany
  - Meta: Llama

##### Low Priority

- Enter system prompts per chat/globally
- Create transparent error handlers for all errors
  - Allow user to re-send a message that was previously sent to a different model
  - learn and use anyhow
- Add a way for users to see descriptions of the models/ link to the docs
- Use local models using TGI interface: https://github.com/huggingface/text-generation-inference
  - or Ollama
  - Refactor and enable local inference + add support for Llama-3-8B locally
- Add image generation APIs
- Add support for agents, such as SWE-Agent: https://github.com/princeton-nlp/SWE-agent
- Include federated learning for local models
- Add streaming API support
- RAG-support
- Word-wrap chat names correctly
- Render markdown tables

### Get started

1. Install Node.js
2. Install Rust
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Commands

- `npm run dev`: Start app in dev mode. It sets `DEVELOPMENT=1` to tell the app to use `./src-tauri/appdata` for app data.
- `npm run build`: Build
- `npm run lint`: Lint

### Store API keys for development

Create a .env file with API keys:

- `openai="YOUR_API_KEY"`
- `mistralai="YOUR_API_KEY"`
- `anthropic="YOUR_API_KEY"`

### Release new version

1. Update `CHANGELOG.md`
2. Bump the version number in `src-tauri/Cargo.toml`
3. Run `cargo check` to update `Cargo.lock`
4. Create a git tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it

### Mac App Store Release:

- Full Guide: https://thinkgo.io/post/2023/02/publish_tauri_to_apples_app_store/
- To release the app, use the provisioning profile from your keychain `Apple Distribution: FRIEDEMANN LEONHARD MANUEL SCHESTAG (UYEQWATXYM)` and copy it into the app bundle under `src-tauri/target/release/bundle/macos/byok.app/Contents/embedded.provisionprofile`
- App Store Connect: https://appstoreconnect.apple.com/apps/6664399508/distribution/macos/version/inflight
- Apple Developer Page: https://developer.apple.com/account/resources/profiles/list

### Acknowledgement

Yoinked the initial code from https://github.com/probablykasper/kadium
