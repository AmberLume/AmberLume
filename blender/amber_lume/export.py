from . import props

def flatten_object(obj):
    amberlume = obj.amberlume
    role = amberlume.object_type

    obj["amberlume_type"] = role

    if role != props.NONE_ROLE:
        spec = props.type_spec(role)
        if spec is not None:
            group = getattr(amberlume, props.pointer_attr(role))

            for field in spec["fields"]:
                obj[field["key"]] = getattr(group, field["key"])

            if role == "Collider":
                obj["collider_name"] = obj.name

    obj.property_unset("amberlume")
