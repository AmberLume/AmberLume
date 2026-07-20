import bpy
from . import props

class AMBERLUME_PT_object(bpy.types.Panel):
    bl_label = "AmberLume"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = 'object'

    @classmethod
    def poll(cls, context):
        return context.object is not None

    def draw(self, context):
        layout = self.layout
        amberlume = context.object.amberlume

        layout.prop(amberlume, "object_type")

        role = amberlume.object_type
        if role == props.NONE_ROLE:
            return

        spec = props.type_spec(role)
        if spec is None:
            layout.label(text="Unknown type: {}".format(role), icon='ERROR')
            return

        group = getattr(amberlume, props.pointer_attr(role))
        box = layout.box()

        if not spec["fields"]:
            box.label(text="No parameters")
            return

        column = box.column()
        for field in spec["fields"]:
            column.prop(group, field["key"])

def register():
    bpy.utils.register_class(AMBERLUME_PT_object)

def unregister():
    bpy.utils.unregister_class(AMBERLUME_PT_object)
