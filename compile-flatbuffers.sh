#!/bin/bash

echo "Compiling flatbuffers..."
flatc.exe --rust -o alpaca/src/data/flatbuffers alpaca/schemas/mesh.fbs
flatc.exe --rust -o alpaca/src/data/flatbuffers alpaca/schemas/model.fbs

echo ""
echo "Flatbuffers compiled"

read -n 1
