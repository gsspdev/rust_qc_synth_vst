#!/bin/zsh

# Set variables
VST_NAME="RustSineWaveSynth"
DYLIB_PATH="target/release/librust_vst_synth_qc_claude.dylib"
VST_BUNDLE_PATH="target/release/${VST_NAME}.vst"

# Create VST bundle structure
mkdir -p "${VST_BUNDLE_PATH}/Contents/MacOS"

# Copy dylib to VST bundle
cp "${DYLIB_PATH}" "${VST_BUNDLE_PATH}/Contents/MacOS/${VST_NAME}"

# Create Info.plist
cat << EOF > "${VST_BUNDLE_PATH}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>${VST_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.yourcompany.${VST_NAME}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${VST_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CSResourcesFileMapped</key>
    <true/>
</dict>
</plist>
EOF

echo "VST bundle created at ${VST_BUNDLE_PATH}"

