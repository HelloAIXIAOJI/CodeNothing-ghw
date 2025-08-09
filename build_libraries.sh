#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Determine build profile
if [ "$1" == "release" ]; then
    BUILD_PROFILE="release"
    CARGO_FLAGS="--release"
    echo "Starting RELEASE build..."
else
    BUILD_PROFILE="debug"
    CARGO_FLAGS=""
    echo "Starting DEBUG build..."
fi

# Build the main project
echo "Building the main project..."
cargo build $CARGO_FLAGS

# Define all the library projects
libraries=(
    "library_io"
    "library_common"
    "library_example"
    "library_os"
    "library_time"
    "library_http"
    "library_fs"
    "library_json"
    "library_math"
)

# Create the target directory for libraries
TARGET_LIB_DIR="./target/$BUILD_PROFILE/library"
if [ ! -d "$TARGET_LIB_DIR" ]; then
    echo "Creating directory: $TARGET_LIB_DIR"
    mkdir -p "$TARGET_LIB_DIR"
fi

# Loop through each library, build it, and copy the output
for lib_dir in "${libraries[@]}"; do
    echo "----------------------------------------"
    echo "Building library: $lib_dir"
    
    # Navigate into the library directory
    # Using a subshell to avoid `cd ..`
    (
      cd "./$lib_dir"
      cargo build $CARGO_FLAGS
    )
    
    # Determine the library name from its directory name
    lib_name=${lib_dir/library_/}
    
    # On Linux, shared libraries are named lib<name>.so
    # On macOS, lib<name>.dylib
    # On Windows, <name>.dll
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        SOURCE_FILE="./$lib_dir/target/$BUILD_PROFILE/lib$lib_name.so"
        TARGET_FILE="$TARGET_LIB_DIR/$lib_name.so"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        SOURCE_FILE="./$lib_dir/target/$BUILD_PROFILE/lib$lib_name.dylib"
        TARGET_FILE="$TARGET_LIB_DIR/$lib_name.dylib"
    else
        # Assuming Windows otherwise
        SOURCE_FILE="./$lib_dir/target/$BUILD_PROFILE/$lib_name.dll"
        TARGET_FILE="$TARGET_LIB_DIR/$lib_name.dll"
    fi

    # If the compiled library file exists, copy it to the shared library folder
    if [ -f "$SOURCE_FILE" ]; then
        echo "Copying $SOURCE_FILE to $TARGET_FILE"
        cp "$SOURCE_FILE" "$TARGET_FILE"
    else
        echo "Warning: $SOURCE_FILE not found, skipping copy."
    fi
done

echo "----------------------------------------"
echo "All libraries built successfully."
echo "You can find the compiled libraries in: $TARGET_LIB_DIR" 