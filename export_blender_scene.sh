#!/bin/bash

INPUT_DIR="$HOME/Models/Lume/scenes/"
OUTPUT_DIR="./lume/assets/scenes/"

blender -b -P "./export_scene.py" -- --input "$INPUT_DIR" --output "$OUTPUT_DIR"
