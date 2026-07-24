bl_info = {
    "name": "AmberLume",
    "author": "Nikita Kladov",
    "version": (2, 0),
    "blender": (5, 0, 0),
    "location": "Properties > Object > AmberLume",
    "description": "Typed model parameters for AmberLume",
    "category": "Object",
}

from . import schema
from . import props
from . import operators
from . import panels
from . import overlay

def register():
    descriptor = schema.load_schema()
    props.register(descriptor)
    operators.register()
    panels.register()
    overlay.register()

def unregister():
    overlay.unregister()
    panels.unregister()
    operators.unregister()
    props.unregister()

if __name__ == "__main__":
    register()
