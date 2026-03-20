#!/bin/bash

INPUT_DIR="$HOME/Models/Lume/models/"
OUTPUT_DIR="../target/generated/gltf"

rm -rf $OUTPUT_DIR
blender -b -P "./blend_to_gltf.py" -- --input "$INPUT_DIR" --output "$OUTPUT_DIR" | grep "^>>"
