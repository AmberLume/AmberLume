import bpy
import os
import sys

def process_blend_file(file_path, output_dir):
    bpy.ops.wm.open_mainfile(filepath=file_path)

    bpy.ops.object.select_all(action='DESELECT')
    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')

    found_any = False

    for obj in bpy.data.objects:
        if obj.instance_type == 'COLLECTION' and obj.instance_collection is not None:
            instance_collection = obj.instance_collection

            if instance_collection.library:
                source_file = os.path.splitext(os.path.basename(instance_collection.library.filepath))[0]
                collection_name = instance_collection.name

                obj["asset_file_name"] = source_file
                obj["collection_name"] = collection_name

                obj["physical_body"] = {
                    "body_type": obj["body_type"].capitalize(),
                    "colliders": collect_colliders_from_collection(instance_collection),
                }

                obj.instance_type = 'NONE'
                obj.instance_collection = None

                obj.select_set(True)
                found_any = True
                print(f"  - Found link: {source_file}#{collection_name}")

    if not found_any:
        print(f"  - No external links found in {os.path.basename(file_path)}")
        return

    asset_file_name = os.path.splitext(os.path.basename(file_path))[0]
    export_path = os.path.join(output_dir, asset_file_name + ".glb")

    bpy.ops.export_scene.gltf(
        filepath=export_path,
        export_format='GLB',
        use_selection=True,
        export_extras=True,
        export_apply=True,
        export_materials='NONE',
        export_animations=True,
        export_skins=False,
        export_yup=True,
        export_lights=False,
    )
    print(f"  - Layout exported to: {export_path}")

def get_collider_info(obj):
    collider_shape = obj["collider_shape"]

    match collider_shape:
        case "box":
            shape = {
                "Box": {
                    "size": [
                        obj.dimensions.x,
                        obj.dimensions.z,
                        obj.dimensions.y,
                    ]
                }
            }
        case "sphere":
            shape = {
                "Sphere": {
                    "radius": max(obj.dimensions) / 2.0,
                }
            }
        case _:
            print(f"Unknown collider shape: {collider_shape}")
            return None

    quat = obj.rotation_euler.to_quaternion()

    return {
        "name": obj["collider_name"],
        "shape": shape,
        "position": [
            obj.location.x,
            obj.location.z,
            -obj.location.y,
        ],
        "rotation": [
            quat.x,
            quat.z,
            -quat.y,
            quat.w,
        ],
    }

def collect_colliders_from_collection(collection):
    colliders = []

    for obj in collection.objects:
        if "collider_shape" in obj:
            collider_info = get_collider_info(obj)

            if collider_info:
                colliders.append(collider_info)
                print(f"    - Collider: {obj.name} ({collider_info['name']})")
                print(f"    - Collider rotation {collider_info['rotation']}")

    for child_collection in collection.children:
        colliders.extend(collect_colliders_from_collection(child_collection))

    return colliders

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        input_dir = args[args.index("--input") + 1]
        output_dir = args[args.index("--output") + 1]
    except (ValueError, IndexError):
        print("Usage: blender -b -P export_scene.py -- --input <dir> --output <dir>")
        return

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    blend_files = [f for f in os.listdir(input_dir) if f.endswith(".blend")]

    for filename in blend_files:
        print(f"\nProcessing scene: {filename}...")
        process_blend_file(os.path.join(input_dir, filename), output_dir)

if __name__ == "__main__":
    main()
