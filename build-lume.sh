#!/bin/bash

MODULE_NAME="lume"

echo "Building lume..."
cargo build -p lume

echo ""
echo "Running builder..."
cargo run --bin builder

echo ""
echo "Copying executable..."
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    EXE="$MODULE_NAME.exe"
else
    EXE="$MODULE_NAME"
fi
cp "./target/build/release/$EXE" "./target/distribution/$EXE"

echo ""
echo "Build completed"

read -n 1
