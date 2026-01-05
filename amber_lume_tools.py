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

class MESH_OT_create_amber_collider(bpy.types.Operator):
    bl_idname = "mesh.create_amber_collider"
    bl_label = "Create collider"
    bl_options = {'REGISTER', 'UNDO'}
    
    shape_type: bpy.props.StringProperty()

    def execute(self, context):
        match self.shape_type:
            case 'box':
                bpy.ops.mesh.primitive_cube_add(size=1)
            case 'sphere':
                bpy.ops.mesh.primitive_uv_sphere_add(radius=1)

        obj = context.active_object
        obj.name = f"{self.shape_type}_collider"
        obj.display_type = 'WIRE'
        
        obj["skip_import"] = True
        obj["collider_shape"] = self.shape_type
        obj["collider_name"] = "collider"
        obj["body_type"] = "static"
        
        return {'FINISHED'}

class MESH_OT_set_static_body(bpy.types.Operator):
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
    
class MESH_OT_set_kinematic_body(bpy.types.Operator):
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
    
class MESH_OT_set_dynamic_body(bpy.types.Operator):
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

class VIEW3D_PT_amber_colliders_panel(bpy.types.Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'AmberLume'
    bl_label = 'Collider templates'

    def draw(self, context):
        layout = self.layout
        col = layout.column(align=True)
        
        op_box = col.operator("mesh.create_amber_collider", text="Box collider", icon='MESH_CUBE')
        op_box.shape_type = 'box'
        
        op_sphere = col.operator("mesh.create_amber_collider", text="Sphere collider", icon='MESH_UVSPHERE')
        op_sphere.shape_type = 'sphere'
        
        op_set_static = col.operator("mesh.set_static_body", text="Set static")
        op_set_kinetic = col.operator("mesh.set_kinematic_body", text="Set kinematic")
        op_set_dynamic = col.operator("mesh.set_dynamic_body", text="Set dynamic")


def register():
    bpy.utils.register_class(MESH_OT_create_amber_collider)
    bpy.utils.register_class(MESH_OT_set_static_body)
    bpy.utils.register_class(MESH_OT_set_kinematic_body)
    bpy.utils.register_class(MESH_OT_set_dynamic_body)
    bpy.utils.register_class(VIEW3D_PT_amber_colliders_panel)

def unregister():
    bpy.utils.unregister_class(MESH_OT_create_amber_collider)
    bpy.utils.unregister_class(MESH_OT_set_static_body)
    bpy.utils.unregister_class(MESH_OT_set_kinematic_body)
    bpy.utils.unregister_class(MESH_OT_set_dynamic_body)
    bpy.utils.unregister_class(VIEW3D_PT_amber_colliders_panel)

if __name__ == "__main__":
    register()
