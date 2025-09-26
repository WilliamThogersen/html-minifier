#!/bin/bash

cd rust

cargo build --release

COPIED_FILES=""

if [ -f "target/release/libhtml_minifier_ffi.dylib" ]; then
    cp "target/release/libhtml_minifier_ffi.dylib" ../src/
    COPIED_FILES="$COPIED_FILES libhtml_minifier_ffi.dylib"
fi

if [ -f "target/release/libhtml_minifier_ffi.so" ]; then
    cp "target/release/libhtml_minifier_ffi.so" ../src/
    COPIED_FILES="$COPIED_FILES libhtml_minifier_ffi.so"
fi

for target_dir in target/*/release/; do
    if [ -f "${target_dir}libhtml_minifier_ffi.dylib" ]; then
        cp "${target_dir}libhtml_minifier_ffi.dylib" ../src/
        COPIED_FILES="$COPIED_FILES libhtml_minifier_ffi.dylib"
    fi
    if [ -f "${target_dir}libhtml_minifier_ffi.so" ]; then
        cp "${target_dir}libhtml_minifier_ffi.so" ../src/
        COPIED_FILES="$COPIED_FILES libhtml_minifier_ffi.so"
    fi
done

if [ -n "$COPIED_FILES" ]; then
    echo "Rust shared library built and copied to src/:$COPIED_FILES"
else
    echo "Error: No library files found to copy."
    exit 1
fi