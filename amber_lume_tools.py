bl_info = {
    "name": "AmberLume Tools",
    "author": "Nikita Kladov",
    "version": (1, 0),
    "blender": (3, 0, 0),
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
        obj.name = f"COL_BOX_{type_name}"
        obj.display_type = 'WIRE'
        
        obj["skip_import"] = True
        obj["collider_shape"] = self.shape_type
        obj["collider_name"] = obj.name
        
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
        hull_obj["collider_shape"] = "convex_hull"
        hull_obj["collider_name"] = hull_obj.name

        return {'FINISHED'}

class SetStaticBody(bpy.types.Operator):
    bl_idname = "mesh.set_static_body"
    bl_label = "Set static"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects
        
        if not selected:
            return {'CANCELLED'}
            
        for obj in selected:
            obj["body_type"] = "static"
            
        self.report({'INFO'}, f"body_type: static")
        return {'FINISHED'}
    
class SetKinematicBody(bpy.types.Operator):
    bl_idname = "mesh.set_kinematic_body"
    bl_label = "Set kinematic"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects
        
        if not selected:
            return {'CANCELLED'}
            
        for obj in selected:
            obj["body_type"] = "kinematic"
            
        self.report({'INFO'}, f"body_type: kinematic")
        return {'FINISHED'}
    
class SetDynamicBody(bpy.types.Operator):
    bl_idname = "mesh.set_dynamic_body"
    bl_label = "Set dynamic"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        selected = context.selected_objects
        
        if not selected:
            return {'CANCELLED'}
            
        for obj in selected:
            obj["body_type"] = "dynamic"
            
        self.report({'INFO'}, f"body_type: dynamic")
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
        op_box.shape_type = 'box'

        op_sphere = colliders_column.operator("mesh.create_collider", text="Create collider (Sphere)", icon='MESH_UVSPHERE')
        op_sphere.shape_type = 'sphere'

        colliders_column.operator("mesh.create_convex_hull_collider", text="Create collider (Convex Hull)", icon='MESH_ICOSPHERE')

        layout.separator()

        body_type_column = layout.column(align=True)

        op_set_static = body_type_column.operator("mesh.set_static_body", text="Set static")
        op_set_kinetic = body_type_column.operator("mesh.set_kinematic_body", text="Set kinematic")
        op_set_dynamic = body_type_column.operator("mesh.set_dynamic_body", text="Set dynamic")


def register():
    bpy.utils.register_class(CreateCollider)
    bpy.utils.register_class(CreateConvexHullCollider)
    bpy.utils.register_class(SetStaticBody)
    bpy.utils.register_class(SetKinematicBody)
    bpy.utils.register_class(SetDynamicBody)
    bpy.utils.register_class(CollidersPanel)

def unregister():
    bpy.utils.unregister_class(CreateCollider)
    bpy.utils.unregister_class(CreateConvexHullCollider)
    bpy.utils.unregister_class(SetStaticBody)
    bpy.utils.unregister_class(SetKinematicBody)
    bpy.utils.unregister_class(SetDynamicBody)
    bpy.utils.unregister_class(CollidersPanel)

if __name__ == "__main__":
    register()
