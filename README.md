# Byok

### Todos:

#### Pre-Release

- add App to Mac App Store, Guidelines:
  - Guide: https://thinkgo.io/post/2023/02/publish_tauri_to_apples_app_store/
  - Get Demo API Keys for App Store Review Demo Account
  - App-Review notes
    - Explain non-obvious features
  - Permission to access SQLite DB in Apple Filesystem
    - Apple File System Reference: https://developer.apple.com/support/downloads/Apple-File-System-Reference.pdf
  - App Store Connect: https://developer.apple.com/help/app-store-connect/
  - Check Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
    - Build for screen readers
    - Correctly set tab indices for VoiceOver
  - Ensure App completeness:
    - https://developer.apple.com/app-store/review/guidelines/#2.1
    - https://developer.apple.com/videos/play/tech-talks/10885/
    - No crashes, broken links, placeholder content
  - Handle errors within the LLM providers
    - Include a note saying that LLM providers can be down sometimes
    - Handle such cases gracefully
  - Ensure that using API-Keys to wrap 3rd Party Services is okay.
    - App Review Guidelines §5.2.2
  - Note that no data is collected through the App itself
    - Submit Privacy Policies by OpenAI, Anthropic
    - Also include links to the privacy policies in the API key page
- Distribute through Steam instead:
  - https://store.steampowered.com/sub/163632

#### Post-Release

##### High Priority

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
- Include a way to leave feedback
  - Page that allows issues with voting/commenting functionality
  - Only accessible via link / token that is given from the App

##### Low Priority

- Create a product page: https://developer.apple.com/app-store/product-page/
  - Simple CTA:
    1. Enter API-Keys
    2. Chat with OpenAI, Anthropic etc.
  - Find new name
    - OmniChat
    - stick with GenHub
    - OmniPilot
    - Byok
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
