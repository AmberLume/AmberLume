bl_info = {
    "name": "AmberLume Tools",
    "author": "Nikita Kladov",
    "version": (1, 1),
    "blender": (5, 0, 0),
    "location": "View3D > Sidebar > AmberLume",
    "description": "AmberLume tools",
    "category": "Object",
}

import bpy

class CreateCollider(bpy.types.Operator):
    bl_idname = "mesh.create_collider"
    bl_label = "Create collider"
    bl_options = {'REGISTER', 'UNDO'}

    shape_type: bpy.props.StringProperty()

    def execute(self, context):
        type_name = "undefined"
        match self.shape_type:
            case 'box':
                type_name = "BOX"
                bpy.ops.mesh.primitive_cube_add(size=1)
            case 'sphere':
                type_name = "SPHERE"
                bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5)

        obj = context.active_object
        obj.name = f"COL_{type_name}"
        obj.display_type = 'WIRE'

        obj["collider_shape"] = self.shape_type
        obj["collider_name"] = obj.name
        obj["friction"] = 0.5
        obj["restitution"] = 0.5
        obj["density"] = 1000.0

        return {'FINISHED'}

class CreateConvexHullCollider(bpy.types.Operator):
    bl_idname = "mesh.create_convex_hull_collider"
    bl_label = "Create Convex Hull from selected"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        source = context.active_object

        if source is None or source.type != 'MESH':
            self.report({'ERROR'}, "Select a mesh object first")
            return {'CANCELLED'}

        bpy.ops.object.duplicate(linked=False)

        bpy.ops.object.convert(target='MESH')

        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.mesh.convex_hull()
        bpy.ops.object.mode_set(mode='OBJECT')

        hull_obj = context.active_object
        hull_obj.name = f"COL_CONVEX_HULL_{context.active_object.name}"
        hull_obj.display_type = 'WIRE'

        hull_obj["skip_import"] = True
        hull_obj["collider_shape"] = "ConvexHull"
        hull_obj["collider_name"] = hull_obj.name
        hull_obj["friction"] = 0.5
        hull_obj["restitution"] = 0.5
        hull_obj["density"] = 1000.0

        return {'FINISHED'}

class SetStaticBody(bpy.types.Operator):
    bl_idname = "mesh.set_static_body"
    bl_label = "Set Static"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects

        if not selected:
            return {'CANCELLED'}

        for obj in selected:
            obj["body_type"] = "Static"

        self.report({'INFO'}, f"body_type: Static")
        return {'FINISHED'}

class SetKinematicBody(bpy.types.Operator):
    bl_idname = "mesh.set_kinematic_body"
    bl_label = "Set Kinematic"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects

        if not selected:
            return {'CANCELLED'}

        for obj in selected:
            obj["body_type"] = "Kinematic"

        self.report({'INFO'}, f"body_type: Kinematic")
        return {'FINISHED'}

class SetDynamicBody(bpy.types.Operator):
    bl_idname = "mesh.set_dynamic_body"
    bl_label = "Set Dynamic"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects

        if not selected:
            return {'CANCELLED'}

        for obj in selected:
            obj["body_type"] = "Dynamic"

        self.report({'INFO'}, f"body_type: Dynamic")
        return {'FINISHED'}

class AddColliderParams(bpy.types.Operator):
    bl_idname = "mesh.add_collider_params"
    bl_label = "Add collider params"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects

        if not selected:
            return {'CANCELLED'}

        for obj in selected:
            obj["friction"] = 0.5
            obj["restitution"] = 0.5
            obj["density"] = 1000.0

        self.report({'INFO'}, f"Collider params added")
        return {'FINISHED'}

class CollidersPanel(bpy.types.Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'AmberLume'
    bl_label = 'Collider templates'

    def draw(self, context):
        layout = self.layout

        colliders_column = layout.column(align=True)

        op_box = colliders_column.operator("mesh.create_collider", text="Create collider (Box)", icon='MESH_CUBE')
        op_box.shape_type = 'Box'

        op_sphere = colliders_column.operator("mesh.create_collider", text="Create collider (Sphere)", icon='MESH_UVSPHERE')
        op_sphere.shape_type = 'Sphere'

        colliders_column.operator("mesh.create_convex_hull_collider", text="Create collider (ConvexHull)", icon='MESH_ICOSPHERE')

        layout.separator()

        body_type_column = layout.column(align=True)

        op_set_static = body_type_column.operator("mesh.set_static_body", text="Set Static")
        op_set_kinetic = body_type_column.operator("mesh.set_kinematic_body", text="Set Kinematic")
        op_set_dynamic = body_type_column.operator("mesh.set_dynamic_body", text="Set Dynamic")

        layout.separator()

        collider_params_column = layout.column(align=True)
        op_add_collider_params = collider_params_column.operator("mesh.add_collider_params", text="Add collider params")


def register():
    bpy.utils.register_class(CreateCollider)
    bpy.utils.register_class(CreateConvexHullCollider)
    bpy.utils.register_class(SetStaticBody)
    bpy.utils.register_class(SetKinematicBody)
    bpy.utils.register_class(SetDynamicBody)
    bpy.utils.register_class(AddColliderParams)
    bpy.utils.register_class(CollidersPanel)

def unregister():
    bpy.utils.unregister_class(CreateCollider)
    bpy.utils.unregister_class(CreateConvexHullCollider)
    bpy.utils.unregister_class(SetStaticBody)
    bpy.utils.unregister_class(SetKinematicBody)
    bpy.utils.unregister_class(SetDynamicBody)
    bpy.utils.unregister_class(AddColliderParams)
    bpy.utils.unregister_class(CollidersPanel)

if __name__ == "__main__":
    register()
