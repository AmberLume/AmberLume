import bpy
from bpy.props import EnumProperty, FloatProperty, BoolProperty, StringProperty, PointerProperty

NONE_ROLE = "NONE"

SCHEMA = None

_registered_classes = []
_keepalive = []

def pointer_attr(role):
    return role.lower()

def type_spec(role):
    if SCHEMA is None:
        return None

    return next((spec for spec in SCHEMA["types"] if spec["role"] == role), None)

def _field_property(field):
    kind = field["kind"]
    label = field["label"]
    kind_type = kind["type"]

    if kind_type == "Enum":
        items = [(variant["value"], variant["label"], "") for variant in kind["variants"]]
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

def _build_role_group(spec):
    annotations = {}
    for field in spec["fields"]:
        annotations[field["key"]] = _field_property(field)

    name = "AmberLume{}Props".format(spec["role"])
    return type(name, (bpy.types.PropertyGroup,), {"__annotations__": annotations})

def _build_object_group(role_groups):
    items = [(NONE_ROLE, "None", "")]
    for spec in SCHEMA["types"]:
        items.append((spec["role"], spec["label"], ""))
    _keepalive.append(items)

    annotations = {
        "object_type": EnumProperty(name="Type", items=items, default=NONE_ROLE),
    }
    for spec in SCHEMA["types"]:
        annotations[pointer_attr(spec["role"])] = PointerProperty(type=role_groups[spec["role"]])

    return type("AmberLumeObjectProps", (bpy.types.PropertyGroup,), {"__annotations__": annotations})

def register(schema):
    global SCHEMA
    SCHEMA = schema

    role_groups = {}
    for spec in SCHEMA["types"]:
        role_group = _build_role_group(spec)
        bpy.utils.register_class(role_group)
        _registered_classes.append(role_group)
        role_groups[spec["role"]] = role_group

    object_group = _build_object_group(role_groups)
    bpy.utils.register_class(object_group)
    _registered_classes.append(object_group)

    bpy.types.Object.amberlume = PointerProperty(type=object_group)

def unregister():
    global SCHEMA

    if hasattr(bpy.types.Object, "amberlume"):
        del bpy.types.Object.amberlume

    for role_group in reversed(_registered_classes):
        bpy.utils.unregister_class(role_group)

    _registered_classes.clear()
    _keepalive.clear()
    SCHEMA = None
