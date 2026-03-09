import bpy
import os
import sys
import bmesh

scene_name_extra = "scene_name"

asset_file_name_extra = "asset_file_name"
asset_collection_name_extra = "collection_name"

asset_physical_body_extra = "physical_body"
body_type_extra = "body_type"

collider_shape_extra = "collider_shape"

def log_error(message):
    print(f">> ERROR: {message}")

def normalize_blender_path(path):
    path = path.replace("//", "")
    path = os.path.normpath(path)
    path = os.path.splitext(path)[0]
    parts = path.split(os.sep)
    clean_parts = [p for p in parts if p not in (".", "..", "")]
    return "/".join(clean_parts)

def process_blend_file(file_path, output_dir):
    bpy.ops.wm.open_mainfile(filepath=file_path)

    print(f">> Extracting scenes from: {file_path}")

    found_scenes = 0

    for scene in bpy.data.scenes:
        print(f">> Found scene '{scene.name}'")

        scene_name = scene.get(scene_name_extra, None)

        if scene_name is None:
            log_error(f">> Scene must have extra '{scene_name_extra}'")
            return

        is_collected = collect_scene(scene, scene_name, output_dir)

        if is_collected:
            found_scenes += 1

    print(f">> Collected {found_scenes} scenes from '{file_path}'")

def collect_scene(scene, scene_name, output_dir):
    print(f">> Collecting scene data: {scene_name}")

    bpy.context.window.scene = scene

    bpy.ops.object.select_all(action='DESELECT')
    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode='OBJECT')

    collected_entities = 0

    scene[scene_name_extra] = scene_name

    for obj in scene.objects:
        is_collection = obj.instance_type == 'COLLECTION'
        has_collection_instance = obj.instance_collection is not None

        if is_collection and has_collection_instance:
            instance_collection = obj.instance_collection

            is_library_collection = instance_collection.library

            if is_library_collection:
                normalized_asset_path = normalize_blender_path(instance_collection.library.filepath)

                source_file = normalized_asset_path
                collection_name = instance_collection.name

                obj[asset_file_name_extra] = source_file
                obj[asset_collection_name_extra] = collection_name

                obj[asset_physical_body_extra] = collect_physical_body_extra(obj, instance_collection)

                obj.instance_type = 'NONE'
                obj.instance_collection = None

                obj.select_set(True)
                collected_entities += 1
            else:
                log_error(f">> Found non-library collection: {instance_collection.name}")

    print(f">> Found {collected_entities} entities in '{scene_name}' scene")

    if collected_entities == 0:
        log_error(f"Scene must contain at least one entity!")

        return False

    export_path = os.path.join(output_dir, scene_name + ".gltf")

    bpy.ops.export_scene.gltf(
        filepath=export_path,
        export_format='GLTF_SEPARATE',
        use_selection=True,
        export_extras=True,
        export_apply=True,
        export_materials='NONE',
        export_animations=True,
        export_skins=False,
        export_yup=True,
        export_lights=False,
    )
    print(f">> Scene exported to {export_path}")

    return True

def collect_physical_body_extra(obj, instance_collection):
    body_type = obj.get(body_type_extra, None)

    if body_type is None:
        log_error(f">> Entity must have '{body_type_extra}' extra! Object name: {obj.name}")
        return None

    colliders = collect_colliders_from_collection(instance_collection)

    if len(colliders) == 0:
        log_error(f">> Physical body must contain at least one collider!")
        return None

    return {
        "body_type": body_type.capitalize(),
        "colliders": colliders,
    }

def collect_colliders_from_collection(collection):
    colliders = []

    for obj in collection.objects:
        if not is_collider(obj):
            continue

        collider_info = collect_collider_info(obj)

        if collider_info is not None:
            colliders.append(collider_info)
        else:
            log_error(f">> Failed to collect collider info: {obj.name}")

    for child_collection in collection.children:
        child_colliders = collect_colliders_from_collection(child_collection)

        colliders.extend(child_colliders)

    return colliders

def is_collider(obj):
    collider_shape = obj.get(collider_shape_extra, None)

    return collider_shape is not None

def collect_collider_info(obj):
    collider_shape = obj.get(collider_shape_extra, None)
    collider_shape = build_shape_from(obj, collider_shape)
    if collider_shape is None:
        log_error(f">> Colliders must have valid `{collider_shape_extra}` extra. Object: {obj.name}")

    quat = obj.rotation_euler.to_quaternion()

    return {
        "name": obj.name,
        "shape": collider_shape,
        "position": [
            obj.location.x,
            obj.location.z,
            -obj.location.y,
        ],
        "scale": [
            obj.scale.x,
            obj.scale.z,
            obj.scale.y,
        ],
        "rotation": [
            quat.x,
            quat.z,
            -quat.y,
            quat.w,
        ],
    }

def build_shape_from(obj, collider_shape):
    match collider_shape:
        case "box":
            return build_box_shape(obj)
        case "sphere":
            return build_sphere_shape(obj)
        case "convex_hull":
            return build_convex_hull_shape(obj)
        case _:
            log_error(f">> Unknown collider shape: {collider_shape}")

            return None

def build_box_shape(obj):
    return {
        "Box": {
            "size": [
                obj.dimensions.x,
                obj.dimensions.z,
                obj.dimensions.y,
            ]
        }
    }

def build_sphere_shape(obj):
    return {
        "Sphere": {
            "radius": max(obj.dimensions) / 2.0,
        }
    }

def build_convex_hull_shape(obj):
    mesh = obj.data
    bm = bmesh.new()
    bm.from_mesh(mesh)

    vertices = []
    for v in bm.verts:
        vertices.append([
            v.co.x,
            v.co.z,
            -v.co.y,
        ])

    bm.free()

    return {
        "ConvexHull": {
            "vertices": vertices,
        }
    }

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        input_dir = args[args.index("--input") + 1]
        output_dir = args[args.index("--output") + 1]
    except (ValueError, IndexError):
        log_error(">> Usage: blender -b -P export_scene.py -- --input <dir> --output <dir>")
        return

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

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

if __name__ == "__main__":
    main()
