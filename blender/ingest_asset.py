import bpy
import os
import sys
import tempfile
from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import amber_lume

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS_DIR = os.path.join(REPO_ROOT, "lume", "core", "resources", "assets")

SKELETON_DIR = "skeleton"
CHARACTER_DIR = "character"
ANIMATIONS_DIR = "animations"

ROOT_BONE = "root"
ROOT_TAIL = (0.0, 0.0, 0.5)
MESH_NAME = "mesh"
COLLIDER_STUB_NAME = "TODO_COLLIDER"

def log(msg):
    print(f">> {msg}")

def log_error(msg):
    print(f">> ERROR: {msg}")

def save(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=path)

def new_file(path, fps, fps_base):
    bpy.ops.wm.read_homefile(use_empty=True)

    render = bpy.context.scene.render
    render.fps = fps
    render.fps_base = fps_base

    save(path)

def find_armature():
    armatures = [obj for obj in bpy.data.objects if obj.type == 'ARMATURE']

    if len(armatures) != 1:
        raise RuntimeError(f"Expected exactly one armature, found {len(armatures)}")

    return armatures[0]

def find_skinned_meshes(armature):
    return [
        obj for obj in bpy.data.objects
        if obj.type == 'MESH' and any(modifier.type == 'ARMATURE' and modifier.object == armature for modifier in obj.modifiers)
    ]

def ensure_root_bone(armature):
    bpy.context.view_layer.objects.active = armature
    bpy.ops.object.mode_set(mode='EDIT')

    edit_bones = armature.data.edit_bones
    tops = [bone for bone in edit_bones if bone.parent is None]

    if [bone.name for bone in tops] != [ROOT_BONE]:
        root = edit_bones.new(ROOT_BONE)
        root.head = (0.0, 0.0, 0.0)
        root.tail = ROOT_TAIL
        root.use_deform = False

        for bone in tops:
            bone.parent = root

    bpy.ops.object.mode_set(mode='OBJECT')

def reset_pose(armature):
    for pose_bone in armature.pose.bones:
        pose_bone.matrix_basis = Matrix.Identity(4)

def upstream_nodes(node):
    found = {node}
    pending = [node]

    while pending:
        for socket in pending.pop().inputs:
            for link in socket.links:
                if link.from_node not in found:
                    found.add(link.from_node)
                    pending.append(link.from_node)

    return found

def normalize_material(material):
    nodes = material.node_tree.nodes

    output = next((
        node for node in nodes
        if node.type == 'OUTPUT_MATERIAL'
        and node.inputs['Surface'].is_linked
        and node.inputs['Surface'].links[0].from_node.type == 'BSDF_PRINCIPLED'
    ), None)

    if output is None:
        return

    output.target = 'ALL'
    output.is_active_output = True

    used = upstream_nodes(output)

    for node in list(nodes):
        if node not in used:
            nodes.remove(node)

def normalize(armature, meshes, name):
    ensure_root_bone(armature)

    for mesh in meshes:
        mesh.name = MESH_NAME

    armature.name = name
    armature.data.name = name

    for action in bpy.data.actions:
        action.use_fake_user = True

    armature.animation_data_clear()
    reset_pose(armature)

    materials = {slot.material for mesh in meshes for slot in mesh.material_slots if slot.material is not None}

    for material in materials:
        normalize_material(material)

def link_skeleton(skeleton_path, name):
    with bpy.data.libraries.load(skeleton_path, link=True, relative=True) as (_, linked):
        linked.objects = [name]

    return linked.objects[0].override_hierarchy_create(bpy.context.scene, bpy.context.view_layer, do_fully_editable=True)

def write_skeleton(armature, path):
    for obj in list(bpy.data.objects):
        if obj != armature:
            bpy.data.objects.remove(obj)

    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)

    for owner in list(armature.users_collection):
        owner.objects.unlink(armature)

    bpy.context.scene.collection.objects.link(armature)

    for collection in list(bpy.data.collections):
        bpy.data.collections.remove(collection)

    armature.amberlume.object_type = 'Skeleton'

    bpy.data.orphans_purge(do_local_ids=True, do_linked_ids=False, do_recursive=True)

    save(path)

def write_animation(action_name, normalized_path, skeleton_path, name, path, fps, fps_base):
    new_file(path, fps, fps_base)

    armature = link_skeleton(skeleton_path, name)

    with bpy.data.libraries.load(normalized_path) as (_, appended):
        appended.actions = [action_name]

    action = appended.actions[0]
    action.use_fake_user = False

    animation_data = armature.animation_data_create()
    animation_data.action = action
    animation_data.action_slot = action.slots[0]

    scene = bpy.context.scene
    scene.frame_start, scene.frame_end = (int(frame) for frame in action.frame_range)

    save(path)

def write_character(mesh_names, normalized_path, skeleton_path, name, path, fps, fps_base):
    new_file(path, fps, fps_base)

    scene = bpy.context.scene

    character = bpy.data.collections.new(name)
    scene.collection.children.link(character)

    armature = link_skeleton(skeleton_path, name)
    scene.collection.objects.unlink(armature)
    character.objects.link(armature)

    with bpy.data.libraries.load(normalized_path) as (_, appended):
        appended.objects = mesh_names

    replaced = set()

    for mesh in appended.objects:
        character.objects.link(mesh)
        mesh.parent = armature

        for modifier in mesh.modifiers:
            if modifier.type == 'ARMATURE':
                replaced.add(modifier.object)
                modifier.object = armature

        mesh.amberlume.object_type = 'Mesh'

    for source in replaced:
        bpy.data.objects.remove(source)

    stub = bpy.data.objects.new(COLLIDER_STUB_NAME, None)
    character.objects.link(stub)
    stub.hide_set(True)

    bpy.data.orphans_purge(do_local_ids=True, do_linked_ids=False, do_recursive=True)

    save(path)

def main():
    try:
        args = sys.argv[sys.argv.index("--") + 1:]
        original_path = os.path.abspath(args[args.index("--original") + 1])
        name = args[args.index("--name") + 1]
    except (ValueError, IndexError):
        log_error("Usage: blender -b -P ingest_asset.py -- --original <file.blend> --name <name>")
        return

    if not hasattr(bpy.types.Object, "amberlume"):
        amber_lume.register()

    skeleton_path = os.path.join(ASSETS_DIR, SKELETON_DIR, name + ".blend")
    character_path = os.path.join(ASSETS_DIR, CHARACTER_DIR, name + ".blend")
    animations_dir = os.path.join(ASSETS_DIR, ANIMATIONS_DIR, name)

    bpy.ops.wm.open_mainfile(filepath=original_path)

    armature = find_armature()
    meshes = find_skinned_meshes(armature)

    normalize(armature, meshes, name)

    mesh_names = [mesh.name for mesh in meshes]

    render = bpy.context.scene.render
    fps, fps_base = render.fps, render.fps_base
    action_names = sorted(action.name for action in bpy.data.actions)

    with tempfile.TemporaryDirectory() as temp_dir:
        normalized_path = os.path.join(temp_dir, os.path.basename(original_path))
        save(normalized_path)

        write_skeleton(armature, skeleton_path)
        log(f"Skeleton: {skeleton_path}")

        for action_name in action_names:
            animation_path = os.path.join(animations_dir, action_name + ".blend")
            write_animation(action_name, normalized_path, skeleton_path, name, animation_path, fps, fps_base)
            log(f"Animation: {animation_path}")

        if os.path.exists(character_path):
            log(f"Character exists, skipped: {character_path}")
        else:
            write_character(mesh_names, normalized_path, skeleton_path, name, character_path, fps, fps_base)
            log(f"Character: {character_path}")

if __name__ == "__main__":
    main()
