import bpy
import os
import sys
import shutil

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def log(msg):
    print(f">> {msg}")

def log_error(msg):
    print(f">> ERROR: {msg}")

# ---------------------------------------------------------------------------
# Linked collection placeholder
#
# Instead of inlining linked geometry we emit an Empty node whose extras
# contain a relative path to the glTF that will be produced for that file.
# The Rust builder resolves this path at build time.
#
# Mapping is simple: foo/bar.blend -> foo/bar.gltf
# ---------------------------------------------------------------------------

def make_linked_ref_empty(linked_collection, current_gltf_abs, input_dir, output_dir, parent_obj=None):
    """Create an Empty node with a relative glTF reference in its extras."""
    linked_blend_abs = os.path.normpath(bpy.path.abspath(linked_collection.library.filepath))

    # foo/bar.blend -> foo/bar.gltf, mirroring input structure into output
    rel_to_input = os.path.relpath(linked_blend_abs, input_dir)
    target_gltf_abs = os.path.join(output_dir, os.path.splitext(rel_to_input)[0] + ".gltf")

    # Relative path from the current gltf to the target gltf
    rel_path = os.path.relpath(target_gltf_abs, os.path.dirname(current_gltf_abs))
    rel_path = rel_path.replace(os.sep, "/")

    empty = bpy.data.objects.new(linked_collection.name, None)
    bpy.context.scene.collection.objects.link(empty)

    empty["amberlume_ref"] = True
    empty["source_gltf"] = rel_path                  # e.g. "../skeleton.gltf"
    empty["source_collection"] = linked_collection.name

    if parent_obj:
        empty.parent = parent_obj

    return empty

# ---------------------------------------------------------------------------
# File export
# ---------------------------------------------------------------------------

def process_blend_file(file_path, input_dir, output_dir):
    """
    Open a .blend file and export the entire scene to a single glTF.
    Linked collections are replaced with Empty placeholder nodes.
    Output path mirrors the input structure: foo/bar.blend -> foo/bar.gltf
    """
    log(f"Processing: {file_path}")

    bpy.ops.wm.open_mainfile(filepath=file_path)

    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='DESELECT')

    rel_to_input = os.path.relpath(file_path, input_dir)
    gltf_abs = os.path.join(output_dir, os.path.splitext(rel_to_input)[0] + ".gltf")
    os.makedirs(os.path.dirname(gltf_abs), exist_ok=True)

    created_empties = []

    # Replace linked collections with Empty placeholder nodes
    for col in bpy.data.collections:
        if col.library:
            empty = make_linked_ref_empty(col, gltf_abs, input_dir, output_dir)
            created_empties.append(empty)

    # Select everything local that is present in the current view layer
    view_layer_objects = {o.name for o in bpy.context.view_layer.objects}
    for obj in bpy.data.objects:
        if obj.library is None and obj.name in view_layer_objects:
            obj.select_set(True)

    bpy.ops.export_scene.gltf(
        filepath=gltf_abs,
        export_format='GLTF_SEPARATE',
        use_selection=True,
        export_extras=True,             # custom properties -> extras
        export_apply=False,             # do not bake transforms
        export_yup=True,                # convert Blender Z-up to glTF Y-up
        export_normals=True,
        export_tangents=True,
        export_materials='EXPORT',
        export_keep_originals=True,
        export_rest_position_armature=True,
        export_skins=True,
        export_animations=True,
    )

    log(f"Exported: {gltf_abs}")

    for empty in created_empties:
        bpy.data.objects.remove(empty, do_unlink=True)

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

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

    log(f"Scanning: {input_dir}")

    for root, dirs, files in os.walk(input_dir):
        for filename in files:
            if filename.endswith(".blend"):
                process_blend_file(os.path.join(root, filename), input_dir, output_dir)

        # Copy images as-is — textures are referenced by glTF and need to be present
        for filename in files:
            if filename.lower().endswith((".png", ".jpg", ".jpeg")):
                dest_dir = os.path.join(output_dir, os.path.relpath(root, input_dir))
                os.makedirs(dest_dir, exist_ok=True)
                shutil.copy(os.path.join(root, filename), dest_dir)
                log(f"Copied image: {filename}")

if __name__ == "__main__":
    main()