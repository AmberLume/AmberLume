import bpy
from bpy.props import EnumProperty, FloatProperty, BoolProperty, StringProperty, PointerProperty

NONE_ROLE = "NONE"
NONE_ROLE_ID = 0
COMPONENT_ROLE = "Placeholder"
COLLIDER_ROLE = "Collider"

SCHEMA = None

_registered_classes = []
_keepalive = []

def pointer_attr(role):
    return role.lower()

def component_attr(name):
    return "component_{}".format(name.lower())

def type_spec(role):
    if SCHEMA is None:
        return None

    return next((spec for spec in SCHEMA["types"] if spec["name"] == role), None)

def component_specs():
    if SCHEMA is None:
        return []

    return SCHEMA.get("components", [])

def _field_property(field):
    kind = field["kind"]
    label = field["label"]
    kind_type = kind["type"]

    if kind_type == "Enum":
        items = [(variant["value"], variant["label"], "", variant["id"]) for variant in kind["variants"]]
        _keepalive.append(items)
        return EnumProperty(name=label, items=items, default=kind["default"])

    if kind_type == "Float":
        kwargs = {"name": label, "default": kind["default"]}
        for bound in ("min", "max", "soft_min", "soft_max"):
            if kind.get(bound) is not None:
                kwargs[bound] = kind[bound]
        return FloatProperty(**kwargs)

    if kind_type == "Bool":
        return BoolProperty(name=label, default=kind["default"])

    if kind_type == "Text":
        return StringProperty(name=label, default=kind["default"])

    raise RuntimeError("unknown field kind {}".format(kind_type))

def _field_annotations(spec):
    annotations = {}

    for field in spec["fields"]:
        annotations[field["key"]] = _field_property(field)

    return annotations

def _build_role_group(spec):
    name = "AmberLume{}Props".format(spec["name"])

    return type(name, (bpy.types.PropertyGroup,), {"__annotations__": _field_annotations(spec)})

def _build_component_group(spec):
    annotations = {"enabled": BoolProperty(name=spec["label"], default=False)}
    annotations.update(_field_annotations(spec))

    name = "AmberLume{}Component".format(spec["name"])

    return type(name, (bpy.types.PropertyGroup,), {"__annotations__": annotations})

def _build_object_group(role_groups, component_groups):
    items = [(NONE_ROLE, "None", "", NONE_ROLE_ID)]
    for spec in SCHEMA["types"]:
        items.append((spec["name"], spec["label"], "", spec["id"]))
    _keepalive.append(items)

    annotations = {
        "object_type": EnumProperty(name="Type", items=items, default=NONE_ROLE),
    }

    for spec in SCHEMA["types"]:
        annotations[pointer_attr(spec["name"])] = PointerProperty(type=role_groups[spec["name"]])

    for spec in component_specs():
        annotations[component_attr(spec["name"])] = PointerProperty(type=component_groups[spec["name"]])

    return type("AmberLumeObjectProps", (bpy.types.PropertyGroup,), {"__annotations__": annotations})

def register(schema):
    global SCHEMA
    SCHEMA = schema

    role_groups = {}
    for spec in SCHEMA["types"]:
        role_group = _build_role_group(spec)
        bpy.utils.register_class(role_group)
        _registered_classes.append(role_group)
        role_groups[spec["name"]] = role_group

    component_groups = {}
    for spec in component_specs():
        component_group = _build_component_group(spec)
        bpy.utils.register_class(component_group)
        _registered_classes.append(component_group)
        component_groups[spec["name"]] = component_group

    object_group = _build_object_group(role_groups, component_groups)
    bpy.utils.register_class(object_group)
    _registered_classes.append(object_group)

    bpy.types.Object.amberlume = PointerProperty(type=object_group)

def unregister():
    global SCHEMA

    if hasattr(bpy.types.Object, "amberlume"):
        del bpy.types.Object.amberlume

    for registered in reversed(_registered_classes):
        bpy.utils.unregister_class(registered)

    _registered_classes.clear()
    _keepalive.clear()
    SCHEMA = None
