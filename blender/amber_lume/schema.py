import os
import json

SCHEMA_RELATIVE = os.path.join("target", "generated", "amberlume_schema.json")

def _find_descriptor():
    current = os.path.dirname(os.path.realpath(__file__))

    while True:
        candidate = os.path.join(current, SCHEMA_RELATIVE)
        if os.path.isfile(candidate):
            return candidate

        parent = os.path.dirname(current)
        if parent == current:
            return None

        current = parent

def load_schema():
    path = _find_descriptor()

    if path is None:
        raise RuntimeError("amberlume_schema.json not found; run `cargo run -p builder` to generate it")

    with open(path, "r") as handle:
        return json.load(handle)
