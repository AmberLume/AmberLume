import bmesh
import bpy
import gpu
from bpy.props import BoolProperty
from gpu_extras.batch import batch_for_shader
from mathutils import Matrix, Vector

from . import collider_shape

OVERLAY_PROPERTY = "amberlume_show_colliders"

ACTIVE_COLOR = (1.0, 0.65, 0.1, 1.0)
IDLE_COLOR = (0.15, 0.85, 0.4, 0.7)
LINE_WIDTH = 1.5
BATCH_CACHE_LIMIT = 128

_draw_handle = None
_shader = None
_batch_cache = {}

def shader():
    global _shader

    if _shader is None:
        _shader = gpu.shader.from_builtin('UNIFORM_COLOR')

    return _shader

def hull_lines(obj):
    if obj.data is None or not hasattr(obj.data, "vertices"):
        return []

    mesh = bmesh.new()
    mesh.from_mesh(obj.data)

    result = bmesh.ops.convex_hull(mesh, input=mesh.verts, use_existing_faces=False)

    bmesh.ops.delete(mesh, geom=result["geom_interior"], context='VERTS')
    bmesh.ops.delete(mesh, geom=result["geom_unused"], context='VERTS')

    lines = [
        (tuple(edge.verts[0].co), tuple(edge.verts[1].co))
        for edge in mesh.edges
    ]

    mesh.free()

    return lines

def batch_of(obj, shape):
    key = collider_shape.signature(obj, shape)
    cached = _batch_cache.get(key)

    if cached is not None:
        return cached

    if shape["type"] == collider_shape.CONVEX_HULL:
        lines = hull_lines(obj)
    else:
        lines = collider_shape.wireframe(shape)

    if not lines:
        return None

    coordinates = [Vector(point) for line in lines for point in line]
    batch = batch_for_shader(shader(), 'LINES', {"pos": coordinates})

    if len(_batch_cache) >= BATCH_CACHE_LIMIT:
        _batch_cache.clear()

    _batch_cache[key] = batch

    return batch

def shape_matrix(obj):
    translation, rotation, _ = obj.matrix_world.decompose()

    return Matrix.Translation(translation) @ rotation.to_matrix().to_4x4()

def draw():
    context = bpy.context

    if not getattr(context.scene, OVERLAY_PROPERTY, False):
        return

    active = shader()

    gpu.state.blend_set('ALPHA')
    gpu.state.depth_test_set('LESS_EQUAL')
    gpu.state.line_width_set(LINE_WIDTH)

    for obj in context.view_layer.objects:
        if not obj.visible_get():
            continue

        shape = collider_shape.shape_of(obj)

        if shape is None:
            continue

        batch = batch_of(obj, shape)

        if batch is None:
            continue

        active.bind()
        active.uniform_float("color", ACTIVE_COLOR if obj.select_get() else IDLE_COLOR)

        with gpu.matrix.push_pop():
            gpu.matrix.multiply_matrix(shape_matrix(obj))
            batch.draw(active)

    gpu.state.line_width_set(1.0)
    gpu.state.depth_test_set('NONE')
    gpu.state.blend_set('NONE')

def on_depsgraph_update(scene, depsgraph):
    for update in depsgraph.updates:
        if update.is_updated_geometry:
            _batch_cache.clear()

            return

def register():
    setattr(bpy.types.Scene, OVERLAY_PROPERTY, BoolProperty(name="Show Colliders", default=True))

    if bpy.app.background:
        return

    global _draw_handle

    _draw_handle = bpy.types.SpaceView3D.draw_handler_add(draw, (), 'WINDOW', 'POST_VIEW')
    bpy.app.handlers.depsgraph_update_post.append(on_depsgraph_update)

def unregister():
    global _draw_handle, _shader

    if _draw_handle is not None:
        bpy.types.SpaceView3D.draw_handler_remove(_draw_handle, 'WINDOW')
        _draw_handle = None

    if on_depsgraph_update in bpy.app.handlers.depsgraph_update_post:
        bpy.app.handlers.depsgraph_update_post.remove(on_depsgraph_update)

    if hasattr(bpy.types.Scene, OVERLAY_PROPERTY):
        delattr(bpy.types.Scene, OVERLAY_PROPERTY)

    _batch_cache.clear()
    _shader = None
