<!-- # Plan -->
<!-- - Remove entity component index redirection thing or else it bungs up the systems
    - Create a systems manager and hard-code rendering system 
        - Run function that receives references to transform and mesh components
        - systems manager is in charge of providing those each frame -->
<!-- - Message/event system using rust enums to pass additional arguments for each event type
    - Message queue that is pumped each frame
    - Maybe allow closures somehow to ease inter-system communication? -->

# Setup keyboard and mouse controls to have a basic look around
- Maybe have some UI widget to control camera parameters like FOV, near/far planes, etc.
# Setup basic objects for grid and coordinate axes
# Get some better models and obj/gltf parsing
- Show coordinates on screen
# I think I'll need wasm-bindgen-futures
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