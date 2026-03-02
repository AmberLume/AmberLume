import shutil

import bpy
import os
import sys

def remove_colliders():
    for obj in bpy.data.objects:
        collider_shape = obj.get("collider_shape")

        if collider_shape is not None:
            bpy.data.objects.remove(obj, do_unlink=True)

def collect_collection(collection, file_export_path):
    collection_name = collection.name

    print(f">> Found collection: {collection_name}")

    root_node = bpy.data.objects.new(collection.name, None)
    bpy.context.scene.collection.objects.link(root_node)

    bpy.ops.object.select_all(action='DESELECT')
    root_node.select_set(True)

    collection_meshes = 0
    for obj in collection.objects:
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)

        bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

        if obj.parent is None:
            obj.parent = root_node

        collection_meshes += 1

    print(f">> Collected {collection_meshes} meshes")

    collection_export_path = os.path.join(file_export_path, collection_name + ".gltf")

    bpy.ops.export_scene.gltf(
        filepath=collection_export_path,
        export_format='GLTF_SEPARATE',
        export_normals=True,
        export_tangents=True,
        use_selection=True,
        export_extras=True,
        export_yup=True,
        export_materials='EXPORT',
        export_keep_originals=True,
        export_rest_position_armature=True,
    )

    for obj in collection.objects:
        if obj.type == 'MESH':
            obj.parent = None
            obj.select_set(False)

    bpy.data.objects.remove(root_node, do_unlink=True)

    bpy.context.view_layer.update()

    print(f">> Exported to: {collection_export_path}")

    return True

def process_blend_file(file_path, output_dir):
    print(f">> ")
    print(f">> Processing {file_path}...")

    bpy.ops.wm.open_mainfile(filepath=file_path)

    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='DESELECT')

    asset_file_name = os.path.splitext(os.path.basename(file_path))[0]
    file_export_path = os.path.join(output_dir, asset_file_name)

    remove_colliders()

    collected_collections = 0

    for collection in bpy.data.collections:
        is_collected = collect_collection(collection, file_export_path)

        if is_collected:
            collected_collections += 1

    print(f">> Collected collections: {collected_collections}")

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        if "--input" not in args or "--output" not in args:
            raise ValueError
        input_dir = args[args.index("--input") + 1]
        output_dir = args[args.index("--output") + 1]
    except (ValueError, IndexError):
        print(">> Usage: blender -b -P export_assets.py -- --input <dir> --output <dir>")
        return

    input_dir = os.path.abspath(input_dir)
    output_dir = os.path.abspath(output_dir)

    if not os.path.exists(input_dir):
        print(f">> Error: Input directory {input_dir} not found")
        return

    print(f">> Scanning {input_dir} recursively...")

    for root, dirs, files in os.walk(input_dir):
        blend_files = [f for f in files if f.endswith(".blend")]

        if not blend_files:
            continue

        relative_path = os.path.relpath(root, input_dir)
        current_output_dir = os.path.join(output_dir, relative_path)

        if not os.path.exists(current_output_dir):
            os.makedirs(current_output_dir)

        for filename in blend_files:
            full_input_path = os.path.join(root, filename)

            process_blend_file(full_input_path, current_output_dir)

    for root, dirs, files in os.walk(input_dir):
        image_files = []

        for file in files:
            is_image = file.endswith(".png") | file.endswith(".jpg")

            if is_image:
                image_files.append(file)

        relative_path = os.path.relpath(root, input_dir)
        current_output_dir = os.path.join(output_dir, relative_path)

        if len(image_files) == 0:
            continue

        if not os.path.exists(current_output_dir):
            os.makedirs(current_output_dir)

        for file in image_files:
            src_path = os.path.join(root, file)

            shutil.copy(src_path, current_output_dir)

            print(f">> Copied image: {src_path}")

if __name__ == "__main__":
    main()
