#!/bin/bash

INPUT_DIR="$HOME/Models/Lume/models/"
OUTPUT_DIR="./lume/assets/models/"

rm -r $OUTPUT_DIR
blender -b -P "./export_assets.py" -- --input "$INPUT_DIR" --output "$OUTPUT_DIR" | grep "^>>"
