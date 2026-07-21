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

        draw_fields(layout, getattr(amberlume, props.pointer_attr(role)), spec)

        if role == props.COMPONENT_ROLE:
            draw_components(layout, amberlume)

def draw_fields(layout, group, spec):
    box = layout.box()

    if not spec["fields"]:
        box.label(text="No parameters")
        return

    column = box.column()
    for field in spec["fields"]:
        column.prop(group, field["key"])

def draw_components(layout, amberlume):
    specs = props.component_specs()
    if not specs:
        return

    layout.label(text="Components")

    for spec in specs:
        group = getattr(amberlume, props.component_attr(spec["name"]))

        box = layout.box()
        box.prop(group, "enabled", text=spec["label"])

        if not group.enabled:
            continue

        column = box.column()
        for field in spec["fields"]:
            column.prop(group, field["key"])

def register():
    bpy.utils.register_class(AMBERLUME_PT_object)

def unregister():
    bpy.utils.unregister_class(AMBERLUME_PT_object)
