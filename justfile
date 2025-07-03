# Justfile for VSTi development workflow

build:
    cargo build --workspace

# Install all plugins to system VST3 directory
# install: build
#     chmod +x build.sh
#     ./build.sh

# Clean all build artifacts
clean:
    cargo clean

# Run tests
test:
    cargo test --workspace

# Check all code without building
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Run clippy on all code
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Create a new plugin template
# new-plugin name:
#     mkdir -p plugins/{{name}}
#     cp -r plugins/sine-synth/* plugins/{{name}}/
#     sed -i 's/sine-synth/{{name}}/g' plugins/{{name}}/Cargo.toml
#     echo "Created new plugin: {{name}}"
#     echo "Remember to:"
#     echo "1. Add '{{name}}' to workspace members in Cargo.toml"
#     echo "2. Update the plugin struct name and constants in plugins/{{name}}/src/lib.rs"

install-bundle name:
    cargo xtask bundle {{name}} --release

install-all:
    ./install-all-bundles.sh