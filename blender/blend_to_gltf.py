import bpy
import os
import sys
import shutil

def log(msg):
    print(f">> {msg}")

def log_error(msg):
    print(f">> ERROR: {msg}")

def process_collection(col, gltf_abs, input_dir, output_dir):
    for obj in col.objects:
        if obj.instance_type == 'COLLECTION' and obj.instance_collection is not None and obj.instance_collection.library is not None:
            linked_col = obj.instance_collection

            linked_blend_abs = os.path.normpath(bpy.path.abspath(linked_col.library.filepath))
            target_gltf_abs = os.path.join(output_dir, os.path.splitext(os.path.relpath(linked_blend_abs, input_dir))[0] + ".gltf")
            source_gltf_key = os.path.relpath(target_gltf_abs, output_dir).replace(os.sep, "/")

            obj["source_gltf"] = source_gltf_key
            obj["source_collection"] = linked_col.name

            # Detach so glTF does not inline the linked contents
            obj.instance_type = 'NONE'
            obj.instance_collection = None

        obj.select_set(True)

    for child in col.children:
        process_collection(child, gltf_abs, input_dir, output_dir)

def process_blend_file(file_path, input_dir, output_dir):
    log(f"Processing: {file_path}")

    bpy.ops.wm.open_mainfile(filepath=file_path)

    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='DESELECT')

    rel_to_input = os.path.relpath(file_path, input_dir)
    gltf_abs = os.path.join(output_dir, os.path.splitext(rel_to_input)[0] + ".gltf")
    os.makedirs(os.path.dirname(gltf_abs), exist_ok=True)

    process_collection(bpy.context.scene.collection, gltf_abs, input_dir, output_dir)

    bpy.ops.export_scene.gltf(
        filepath=gltf_abs,
        export_format='GLTF_SEPARATE',
        use_selection=True,
        export_extras=True,
        export_apply=False,
        export_yup=True,
        export_normals=True,
        export_tangents=True,
        export_materials='EXPORT',
        export_keep_originals=True,
        export_rest_position_armature=True,
        export_skins=True,
        export_animations=True,
        export_hierarchy_full_collections=True,
    )

    log(f"Exported: {gltf_abs}")

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

        for filename in files:
            if filename.lower().endswith((".png", ".jpg", ".jpeg")):
                dest_dir = os.path.join(output_dir, os.path.relpath(root, input_dir))
                os.makedirs(dest_dir, exist_ok=True)
                shutil.copy(os.path.join(root, filename), dest_dir)
                log(f"Copied image: {filename}")

if __name__ == "__main__":
    main()
