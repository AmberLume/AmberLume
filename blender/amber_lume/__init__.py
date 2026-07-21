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
from . import panels

def register():
    descriptor = schema.load_schema()
    props.register(descriptor)
    panels.register()

def unregister():
    panels.unregister()
    props.unregister()

if __name__ == "__main__":
    register()
