from math import cos, pi, sin

from . import props

BOX = "Box"
CAPSULE = "Capsule"
CONVEX_HULL = "ConvexHull"

RING_SEGMENTS = 32
CAP_SEGMENTS = 8

SHAPE_FIELD = "collider_shape"

def variants():
    spec = props.type_spec(props.COLLIDER_ROLE)

    if spec is None:
        return []

    for field in spec["fields"]:
        if field["key"] == SHAPE_FIELD:
            return [(variant["value"], variant["label"]) for variant in field["kind"]["variants"]]

    return []

def needs_geometry(shape_type):
    return shape_type == CONVEX_HULL

def default_scale(shape_type):
    if shape_type == CAPSULE:
        return (0.6, 0.6, 1.8)

    return (1.0, 1.0, 1.0)

def shape_of(obj):
    amberlume = getattr(obj, "amberlume", None)

    if amberlume is None or amberlume.object_type != props.COLLIDER_ROLE:
        return None

    group = getattr(amberlume, props.pointer_attr(props.COLLIDER_ROLE))

    return derive(group.collider_shape, obj.matrix_world.to_scale())

def derive(shape_type, scale):
    size = (abs(scale[0]), abs(scale[1]), abs(scale[2]))

    if shape_type == BOX:
        return {"type": BOX, "size": size}

    if shape_type == CAPSULE:
        radius = max(size[0], size[1]) / 2.0
        half_height = max(size[2] / 2.0 - radius, 0.0)

        return {"type": CAPSULE, "radius": radius, "half_height": half_height}

    if shape_type == CONVEX_HULL:
        return {"type": CONVEX_HULL}

    return None

def describe(shape):
    if shape["type"] == BOX:
        size = shape["size"]

        return [
            ("Size X", "{:.3f} m".format(size[0])),
            ("Size Y", "{:.3f} m".format(size[1])),
            ("Size Z", "{:.3f} m".format(size[2])),
        ]

    if shape["type"] == CAPSULE:
        radius = shape["radius"]
        half_height = shape["half_height"]

        return [
            ("Radius", "{:.3f} m".format(radius)),
            ("Half Height", "{:.3f} m".format(half_height)),
            ("Total Height", "{:.3f} m".format(2.0 * (half_height + radius))),
            ("Axis", "Z (up)"),
        ]

    return [("Points", "mesh vertices")]

def wireframe(shape):
    if shape["type"] == BOX:
        return box_lines(shape["size"])

    if shape["type"] == CAPSULE:
        return capsule_lines(shape["radius"], shape["half_height"])

    return []

def box_lines(size):
    half = (size[0] / 2.0, size[1] / 2.0, size[2] / 2.0)

    corners = [
        (x * half[0], y * half[1], z * half[2])
        for x in (-1.0, 1.0)
        for y in (-1.0, 1.0)
        for z in (-1.0, 1.0)
    ]

    edges = [
        (0, 1), (2, 3), (4, 5), (6, 7),
        (0, 2), (1, 3), (4, 6), (5, 7),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ]

    return [(corners[start], corners[end]) for start, end in edges]

def capsule_lines(radius, half_height):
    lines = []

    lines.extend(ring_lines(radius, half_height))
    lines.extend(ring_lines(radius, -half_height))
    lines.extend(cap_lines(radius, half_height, 1.0))
    lines.extend(cap_lines(radius, -half_height, -1.0))

    for axis in (0, 1):
        for direction in (1.0, -1.0):
            top = [0.0, 0.0, half_height]
            bottom = [0.0, 0.0, -half_height]

            top[axis] = direction * radius
            bottom[axis] = direction * radius

            lines.append((tuple(top), tuple(bottom)))

    return lines

def ring_lines(radius, height):
    lines = []

    for index in range(RING_SEGMENTS):
        first = 2.0 * pi * index / RING_SEGMENTS
        second = 2.0 * pi * (index + 1) / RING_SEGMENTS

        lines.append((
            (radius * cos(first), radius * sin(first), height),
            (radius * cos(second), radius * sin(second), height),
        ))

    return lines

def cap_lines(radius, height, direction):
    lines = []

    for index in range(CAP_SEGMENTS):
        first = 0.5 * pi * index / CAP_SEGMENTS
        second = 0.5 * pi * (index + 1) / CAP_SEGMENTS

        first_height = height + direction * radius * sin(first)
        second_height = height + direction * radius * sin(second)

        for axis in (0, 1):
            for side in (1.0, -1.0):
                start = [0.0, 0.0, first_height]
                end = [0.0, 0.0, second_height]

                start[axis] = side * radius * cos(first)
                end[axis] = side * radius * cos(second)

                lines.append((tuple(start), tuple(end)))

    return lines

def signature(obj, shape):
    if shape["type"] == BOX:
        return (BOX,) + shape["size"]

    if shape["type"] == CAPSULE:
        return (CAPSULE, shape["radius"], shape["half_height"])

    return (CONVEX_HULL, obj.data.name if obj.data is not None else obj.name)
