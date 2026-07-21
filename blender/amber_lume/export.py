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

        if role == props.COMPONENT_ROLE:
            obj["amberlume_components"] = collect_components(amberlume)

    obj.property_unset("amberlume")

def collect_components(amberlume):
    components = []

    for spec in props.component_specs():
        group = getattr(amberlume, props.component_attr(spec["name"]))

        if not group.enabled:
            continue

        values = {field["key"]: getattr(group, field["key"]) for field in spec["fields"]}

        components.append({spec["name"]: values})

    return components
