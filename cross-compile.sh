#!/bin/bash

cd rust

echo "Building for multiple targets..."

# Build for current platform (default)
echo "Building for current platform..."
cargo build --release

# Build for Linux x86_64 (if on macOS with cross-compilation setup)
if command -v x86_64-linux-gnu-gcc &> /dev/null; then
    echo "Building for Linux x86_64..."
    cargo build --release --target x86_64-unknown-linux-gnu
fi

# Build for Linux ARM64 (if cross-compilation is available)
if command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo "Building for Linux ARM64..."
    cargo build --release --target aarch64-unknown-linux-gnu
fi

# Copy all available libraries
echo "Copying libraries..."
COPIED_FILES=""

# Copy from main target directory (only shared libraries)
for ext in so dylib dll; do
    lib="target/release/libhtml_minifier_ffi.$ext"
    if [ -f "$lib" ]; then
        cp "$lib" ../src/
        COPIED_FILES="$COPIED_FILES $(basename $lib)"
    fi
done

# Copy from cross-compilation target directories (only shared libraries)
for target_dir in target/*/release/; do
    if [ -d "$target_dir" ]; then
        target_name=$(echo "$target_dir" | cut -d'/' -f2)
        for ext in so dylib dll; do
            lib="${target_dir}libhtml_minifier_ffi.$ext"
            if [ -f "$lib" ]; then
                # Copy without target architecture suffix for .so files
                if [ "$ext" = "so" ]; then
                    new_name="libhtml_minifier_ffi.${ext}"
                else
                    new_name="libhtml_minifier_ffi-${target_name}.${ext}"
                fi
                cp "$lib" "../src/${new_name}"
                COPIED_FILES="$COPIED_FILES ${new_name}"
            fi
        done
    fi
done

if [ -n "$COPIED_FILES" ]; then
    echo "Libraries built and copied:$COPIED_FILES"
else
    echo "No libraries were built."
    exit 1
fi