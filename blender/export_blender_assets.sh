#!/bin/bash

INPUT_DIR="../lume/core/resources/assets/"
OUTPUT_DIR="../target/generated/prebuild/assets/"

rm -rf $OUTPUT_DIR
blender -b -P "./blend_to_gltf.py" -- --input "$INPUT_DIR" --output "$OUTPUT_DIR" | grep "^>>"
