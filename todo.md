# Bootstrapping
<!-- - Remove entity component index redirection thing or else it bungs up the systems
    - Create a systems manager and hard-code rendering system 
        - Run function that receives references to transform and mesh components
        - systems manager is in charge of providing those each frame -->
<!-- - Message/event system using rust enums to pass additional arguments for each event type
    - Message queue that is pumped each frame
    - Maybe allow closures somehow to ease inter-system communication? -->
<!-- - Add widgets to UIComponents -->
<!-- - Do materials -->
<!-- - Nothing is really resizing/creating the components yet -->
<!-- - Nothing checks if the components are valid before actually doing stuff with them -->
- Materials should be managed by res_man
- Wire up event manager so that it at least exists and lifetimes are OK and stuff

# Put camera info on app state, make some sort of "main camera" thing
# Setup keyboard and mouse controls to have a basic look around
- Fixup mouse events (once its back to rendering)
# How to deal with object hierarchies?
# Finish up event system
# UI widget to control camera parameters like FOV, near/far planes, etc.
# Setup basic objects for grid and coordinate axes
# Get some better models and obj/gltf parsing
- Show coordinates on screen
# I think I'll need wasm-bindgen-futures at some point for something?
<!-- # Setup a time variable and animate a material rotation -->
<!-- # Why is the cube rendering at the bottom left?
- Missing viewport -->


# Cool sources
- https://github.com/bevyengine/bevy
- https://github.com/not-fl3/macroquad
- https://github.com/hecrj/coffee
- https://github.com/ggez/ggez
- https://github.com/mrDIMAS/rg3d
- https://github.com/PistonDevelopers/piston