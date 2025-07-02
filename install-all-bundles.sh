#!/bin/bash

# Build script for all VSTi plugins
set -e

echo "Building all VSTi plugins..."

# Build all plugins in release mode
# cargo build --release --workspace

# Create VST3 installation directory if it doesn't exist
VST3_DIR=""
case "$(uname -s)" in
    Darwin*)
        VST3_DIR="$HOME/Library/Audio/Plug-Ins/VST3/blight-vsti"
        ;;
    # Linux*)
    #     VST3_DIR="$HOME/.vst3"
    #     LIB_EXT="so"
    #     ;;
    # MINGW*|CYGWIN*|MSYS*)
    #     VST3_DIR="$PROGRAMFILES/Common Files/VST3"
    #     LIB_EXT="dll"
    #     ;;

    *)
        echo "Unknown OS"
        exit 1
        ;;
esac

rm -rf "$VST3_DIR"
mkdir -p "$VST3_DIR"
current_dir=$(pwd)

echo "Working directory: $current_dir"
vsti_dir="$current_dir/target/bundled"

# Install each plugin
for plugin_dir in plugins/*/; do
    plugin_name=$(basename "$plugin_dir")
    plugin_path="$vsti_dir/$plugin_name.vst3"

    just bundle "$plugin_name"
    echo "Processing plugin: $plugin_name located at $plugin_path"
    if [ -d "$plugin_path" ]; then
        echo "Installing $plugin_name to $VST3_DIR..."
        cp -r "$plugin_path" "$VST3_DIR/"
    else
        echo "Warning: $plugin_path not found, skipping $plugin_name"
    fi
done

echo "Build and installation complete!"
echo "Plugins installed to: $VST3_DIR"