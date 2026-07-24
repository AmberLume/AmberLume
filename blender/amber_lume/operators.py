import bmesh
import bpy
from bpy.props import StringProperty
from mathutils import Matrix, Vector

from . import collider_shape
from . import props

COLLIDER_NAME = "collider"
EMPTY_DISPLAY_SIZE = 0.5

class AMBERLUME_OT_add_collider(bpy.types.Operator):
    bl_idname = "amberlume.add_collider"
    bl_label = "Add Collider"
    bl_options = {'REGISTER', 'UNDO'}

    shape: StringProperty(name="Shape", default=collider_shape.BOX)

    def execute(self, context):
        if collider_shape.needs_geometry(self.shape):
            return self.add_hull(context)

        return self.add_empty(context)

    def add_empty(self, context):
        bpy.ops.object.empty_add(type='PLAIN_AXES', location=context.scene.cursor.location)

        obj = context.active_object
        obj.name = COLLIDER_NAME
        obj.empty_display_size = EMPTY_DISPLAY_SIZE
        obj.scale = collider_shape.default_scale(self.shape)

        mark_collider(obj, self.shape)

        return {'FINISHED'}

    def add_hull(self, context):
        source = context.active_object

        if source is None or source.type != 'MESH':
            self.report({'ERROR'}, "Select a mesh object first")

            return {'CANCELLED'}

        translation, rotation, scale = source.matrix_world.decompose()

        mesh = bmesh.new()
        mesh.from_mesh(source.data)

        result = bmesh.ops.convex_hull(mesh, input=mesh.verts, use_existing_faces=False)

        bmesh.ops.delete(mesh, geom=result["geom_interior"], context='VERTS')
        bmesh.ops.delete(mesh, geom=result["geom_unused"], context='VERTS')

        for vertex in mesh.verts:
            vertex.co = Vector((
                vertex.co.x * scale.x,
                vertex.co.y * scale.y,
                vertex.co.z * scale.z,
            ))

        data = bpy.data.meshes.new(COLLIDER_NAME)
        mesh.to_mesh(data)
        mesh.free()

        obj = bpy.data.objects.new(COLLIDER_NAME, data)
        context.collection.objects.link(obj)

        obj.matrix_world = Matrix.Translation(translation) @ rotation.to_matrix().to_4x4()
        obj.display_type = 'WIRE'

        mark_collider(obj, self.shape)

        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        context.view_layer.objects.active = obj

        return {'FINISHED'}

class AMBERLUME_MT_add_collider(bpy.types.Menu):
    bl_idname = "AMBERLUME_MT_add_collider"
    bl_label = "AmberLume Collider"

    def draw(self, context):
        draw_add_operators(self.layout)

def draw_add_operators(layout):
    for value, label in collider_shape.variants():
        layout.operator(AMBERLUME_OT_add_collider.bl_idname, text=label).shape = value

def mark_collider(obj, shape_type):
    obj.amberlume.object_type = props.COLLIDER_ROLE

    group = getattr(obj.amberlume, props.pointer_attr(props.COLLIDER_ROLE))
    group.collider_shape = shape_type

def draw_add_menu(self, context):
    self.layout.menu(AMBERLUME_MT_add_collider.bl_idname, icon='PHYSICS')

def register():
    bpy.utils.register_class(AMBERLUME_OT_add_collider)
    bpy.utils.register_class(AMBERLUME_MT_add_collider)

    bpy.types.VIEW3D_MT_add.append(draw_add_menu)

def unregister():
    bpy.types.VIEW3D_MT_add.remove(draw_add_menu)

    bpy.utils.unregister_class(AMBERLUME_MT_add_collider)
    bpy.utils.unregister_class(AMBERLUME_OT_add_collider)
