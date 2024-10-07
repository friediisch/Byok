APP_NAME="byok"
PROJECT_DIR="."

APPLE_DISTRIBUTION_CERT="Apple Distribution: FRIEDEMANN LEONHARD MANUEL SCHESTAG (UYEQWATXYM)"
THIRD_PARTY_MAC_DEVELOPER_CERT="3rd Party Mac Developer Installer: FRIEDEMANN LEONHARD MANUEL SCHESTAG (UYEQWATXYM)"

# Change if needed:

APP_BUNDLE="$APP_NAME.app"
APP_EXECUTABLE="$APP_BUNDLE/Contents/MacOS/$APP_NAME"
APP_PACKAGE="$APP_NAME.pkg"

# Build, Sign, Package, Upload:

npm run tauri build --release -- --target universal-apple-darwin

rm -rf "$APP_BUNDLE"

cp -r "$PROJECT_DIR/src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_BUNDLE" .

# re-enter the embedded.provisionprofile
cp "embedded.provisionprofile" "$APP_BUNDLE/Contents/embedded.provisionprofile"

codesign \
	--sign "$APPLE_DISTRIBUTION_CERT" \
	--entitlements "entitlements.plist" \
	"$APP_EXECUTABLE"

rm -rf "$APP_PACKAGE"

productbuild \
	--sign "$THIRD_PARTY_MAC_DEVELOPER_CERT" \
	--component "$APP_BUNDLE" "/Applications" \
	"$APP_PACKAGE"