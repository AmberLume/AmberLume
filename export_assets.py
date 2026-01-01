import bpy
import os
import sys

def process_blend_file(file_path, output_dir):
    bpy.ops.wm.open_mainfile(filepath=file_path)

    asset_file_name = os.path.splitext(os.path.basename(file_path))[0]

    export_path = os.path.join(output_dir, asset_file_name + ".glb")

    for collection in bpy.data.collections:
        root_node = bpy.data.objects.new(collection.name, None)
        bpy.context.scene.collection.objects.link(root_node)

        for obj in collection.objects:
            if obj.parent is None:
                obj.parent = root_node

    bpy.ops.export_scene.gltf(
        filepath=export_path,
        export_format='GLB',
        export_extras=True,
        export_apply=True,
        export_materials='EXPORT',
        export_rest_position_armature=True,
    )

    print(f"  - Exported to: {export_path}")

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        if "--input" not in args or "--output" not in args:
            raise ValueError
        input_dir = args[args.index("--input") + 1]
        output_dir = args[args.index("--output") + 1]
    except (ValueError, IndexError):
        print("Usage: blender -b -P export_assets.py -- --input <dir> --output <dir>")
        return

    if not os.path.exists(input_dir):
        print(f"Error: Input directory {input_dir} not found")
        return

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    blend_files = [f for f in os.listdir(input_dir) if f.endswith(".blend")]

    if not blend_files:
        print(f"No .blend files found in {input_dir}")
        return

    print(f"Found {len(blend_files)} files to process.")

    for filename in blend_files:
        full_path = os.path.join(input_dir, filename)
        print(f"\nProcessing: {filename}...")
        try:
            process_blend_file(full_path, output_dir)
        except Exception as e:
            print(f"Failed to process {filename}: {e}")

if __name__ == "__main__":
    main()
