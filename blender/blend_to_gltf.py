import bpy
import os
import sys
import shutil
import json
import hashlib

CACHE_FILENAME = ".cache.json"
SCRIPT_HASH_KEY = "__script_hash__"

def log(msg):
    print(f">> {msg}")

def log_error(msg):
    print(f">> ERROR: {msg}")

ADDON_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "amber_lume")
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DESCRIPTOR_PATH = os.path.join(REPO_ROOT, "target", "generated", "amberlume_schema.json")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import amber_lume
    from amber_lume import export as amber_export
    if not hasattr(bpy.types.Object, "amberlume"):
        amber_lume.register()
    AMBERLUME_READY = True
except Exception as error:
    log_error(f"AmberLume addon unavailable, exporting without typed params: {error}")
    AMBERLUME_READY = False

def file_hash(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

def combined_hash(paths):
    h = hashlib.sha256()
    for path in sorted(paths):
        h.update(path.encode("utf-8"))
        if os.path.isfile(path):
            with open(path, "rb") as f:
                for chunk in iter(lambda: f.read(65536), b""):
                    h.update(chunk)
        else:
            h.update(b"<missing>")
    return h.hexdigest()

def script_inputs():
    inputs = [os.path.abspath(__file__), DESCRIPTOR_PATH]
    if os.path.isdir(ADDON_DIR):
        for name in os.listdir(ADDON_DIR):
            if name.endswith(".py"):
                inputs.append(os.path.join(ADDON_DIR, name))
    return inputs

def load_cache(cache_path):
    if not os.path.exists(cache_path):
        return {}
    try:
        with open(cache_path, "r") as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return {}

def save_cache(cache_path, cache):
    os.makedirs(os.path.dirname(cache_path), exist_ok=True)
    with open(cache_path, "w") as f:
        json.dump(cache, f, indent=2, sort_keys=True)

def to_rel(abs_path, base):
    return os.path.relpath(abs_path, base).replace(os.sep, "/")

def all_outputs_present(output_dir, outputs):
    return all(os.path.exists(os.path.join(output_dir, o)) for o in outputs)

def process_collection(collection, gltf_abs, input_dir, output_dir):
    for obj in collection.objects:
        if obj.instance_type == 'COLLECTION' and obj.instance_collection is not None and obj.instance_collection.library is not None:
            reference = obj.instance_collection
            library = reference.library

            set_link_extras(obj, reference, library, gltf_abs, input_dir, output_dir)

            obj.instance_type = 'NONE'
            obj.instance_collection = None

        if obj.override_library is not None:
            reference = collection.override_library.reference
            library = reference.library

            set_link_extras(collection, reference, library, gltf_abs, input_dir, output_dir)

        obj.select_set(True)

    for child in collection.children:
        process_collection(child, gltf_abs, input_dir, output_dir)

def set_link_extras(obj, reference, library, gltf_abs, input_dir, output_dir):
    linked_blend_abs = os.path.normpath(bpy.path.abspath(library.filepath))
    target_gltf_abs = os.path.join(output_dir, os.path.splitext(os.path.relpath(linked_blend_abs, input_dir))[0] + ".gltf")

    gltf_dir = os.path.dirname(gltf_abs)
    source_gltf_key = os.path.relpath(target_gltf_abs, gltf_dir).replace(os.sep, "/")

    obj["source_gltf"] = source_gltf_key
    obj["source_collection"] = reference.name

def export_blend(file_path, input_dir, output_dir):
    log(f"Exporting: {file_path}")

    bpy.ops.wm.open_mainfile(filepath=file_path)

    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='DESELECT')

    rel_to_input = os.path.relpath(file_path, input_dir)
    gltf_abs = os.path.join(output_dir, os.path.splitext(rel_to_input)[0] + ".gltf")
    os.makedirs(os.path.dirname(gltf_abs), exist_ok=True)

    process_collection(bpy.context.scene.collection, gltf_abs, input_dir, output_dir)

    if AMBERLUME_READY:
        for obj in bpy.context.scene.objects:
            amber_export.flatten_object(obj)

    bpy.ops.export_scene.gltf(
        filepath=gltf_abs,
        export_format='GLTF_SEPARATE',
        use_selection=True,
        export_extras=True,
        export_apply=False,
        export_rest_position_armature=False,
        export_yup=True,
        export_normals=True,
        export_tangents=True,
        export_materials='EXPORT',
        export_keep_originals=True,
        export_skins=True,
        export_animations=True,
        export_hierarchy_full_collections=True,
    )

    outputs = []
    if os.path.exists(gltf_abs):
        outputs.append(to_rel(gltf_abs, output_dir))
    bin_abs = os.path.splitext(gltf_abs)[0] + ".bin"
    if os.path.exists(bin_abs):
        outputs.append(to_rel(bin_abs, output_dir))

    return outputs

def remove_path(path):
    try:
        os.remove(path)
        return True
    except OSError as e:
        log_error(f"Failed to remove {path}: {e}")
        return False

def prune_empty_dirs(root):
    for dirpath, _, _ in os.walk(root, topdown=False):
        if dirpath == root:
            continue
        try:
            os.rmdir(dirpath)
        except OSError:
            pass

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        input_dir = args[args.index("--input") + 1]
        output_dir = args[args.index("--output") + 1]
    except (ValueError, IndexError):
        log_error("Usage: blender -b -P export.py -- --input <dir> --output <dir>")
        return

    input_dir = os.path.abspath(input_dir)
    output_dir = os.path.abspath(output_dir)

    if not os.path.exists(input_dir):
        log_error(f"Input directory not found: {input_dir}")
        return

    os.makedirs(output_dir, exist_ok=True)

    script_hash = combined_hash(script_inputs())
    cache_path = os.path.join(output_dir, CACHE_FILENAME)
    cache = load_cache(cache_path)

    if cache.get(SCRIPT_HASH_KEY) != script_hash:
        log("Script changed, invalidating cache")
        cache = {SCRIPT_HASH_KEY: script_hash, "entries": {}}

    entries = cache.setdefault("entries", {})

    log(f"Scanning: {input_dir}")

    seen_sources = set()
    added = []
    changed = []
    unchanged = []
    failed = []

    for root, _, files in os.walk(input_dir):
        for filename in files:
            full_path = os.path.join(root, filename)
            rel_path = to_rel(full_path, input_dir)
            lower = filename.lower()

            if lower.endswith(".blend"):
                seen_sources.add(rel_path)
                current_hash = file_hash(full_path)
                entry = entries.get(rel_path)

                if entry is not None and entry.get("hash") == current_hash and all_outputs_present(output_dir, entry.get("outputs", [])):
                    unchanged.append(rel_path)
                    continue

                try:
                    outputs = export_blend(full_path, input_dir, output_dir)
                except Exception as e:
                    log_error(f"Failed to export {rel_path}: {e}")
                    failed.append(rel_path)
                    continue

                (added if entry is None else changed).append(rel_path)
                entries[rel_path] = {"hash": current_hash, "outputs": outputs}

            elif lower.endswith((".png", ".jpg", ".jpeg")):
                seen_sources.add(rel_path)
                current_hash = file_hash(full_path)
                entry = entries.get(rel_path)

                dest_dir = os.path.join(output_dir, os.path.relpath(root, input_dir))
                dest_path = os.path.join(dest_dir, filename)
                dest_rel = to_rel(dest_path, output_dir)

                if entry is not None and entry.get("hash") == current_hash and os.path.exists(dest_path):
                    unchanged.append(rel_path)
                    continue

                os.makedirs(dest_dir, exist_ok=True)
                shutil.copy(full_path, dest_path)
                log(f"Copied image: {rel_path}")

                (added if entry is None else changed).append(rel_path)
                entries[rel_path] = {"hash": current_hash, "outputs": [dest_rel]}

    removed = []
    removed_outputs = 0
    for src in list(entries.keys()):
        if src in seen_sources:
            continue
        for out_rel in entries[src].get("outputs", []):
            out_abs = os.path.join(output_dir, out_rel)
            if os.path.exists(out_abs) and remove_path(out_abs):
                removed_outputs += 1
        removed.append(src)
        del entries[src]

    prune_empty_dirs(output_dir)

    save_cache(cache_path, cache)

    log("=== Build diff ===")
    log(f"Added:     {len(added)}")
    for p in added:
        log(f"  + {p}")
    log(f"Changed:   {len(changed)}")
    for p in changed:
        log(f"  ~ {p}")
    log(f"Removed:   {len(removed)} sources, {removed_outputs} output files")
    for p in removed:
        log(f"  - {p}")
    log(f"Unchanged: {len(unchanged)}")
    if failed:
        log(f"Failed:    {len(failed)}")
        for p in failed:
            log(f"  ! {p}")

if __name__ == "__main__":
    main()
