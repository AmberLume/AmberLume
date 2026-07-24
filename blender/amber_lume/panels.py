import bpy
from . import collider_shape
from . import operators
from . import overlay
from . import props

class AMBERLUME_PT_viewport(bpy.types.Panel):
    bl_label = "AmberLume"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'AmberLume'

    def draw(self, context):
        layout = self.layout

        layout.prop(context.scene, overlay.OVERLAY_PROPERTY)
        layout.separator()
        layout.label(text="Add Collider")

        operators.draw_add_operators(layout.column(align=True))

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

        if role == props.COLLIDER_ROLE:
            draw_shape(layout, context.object)

        if role == props.COMPONENT_ROLE:
            draw_components(layout, amberlume)

def draw_shape(layout, obj):
    shape = collider_shape.shape_of(obj)

    if shape is None:
        return

    layout.label(text="Resolved Shape")

    box = layout.box()
    column = box.column(align=True)

    for label, value in collider_shape.describe(shape):
        row = column.row()
        row.label(text=label)
        row.label(text=value)

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
    bpy.utils.register_class(AMBERLUME_PT_viewport)

def unregister():
    bpy.utils.unregister_class(AMBERLUME_PT_viewport)
    bpy.utils.unregister_class(AMBERLUME_PT_object)
