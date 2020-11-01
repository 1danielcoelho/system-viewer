<!-- # Bootstrapping -->
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
<!-- - Materials should be managed by res_man -->
<!-- - Restore UI so I can check keyboard events -->
<!-- - Wire up event manager so that it at least exists and lifetimes are OK and stuff -->
<!-- - I think there is an OBOB on the component/entity count... one too many for what I should have now -->
<!-- - Setup basic mouse and keyboard events -->
<!-- - Fixup mouse events (once its back to rendering) -->
<!-- # I want to move around and see camera data on the UI -->
<!-- - Put camera info on app state -->
<!-- - Make some sort of "main camera" thing on app state
    - Maybe I shouldn't make cameras into "components" as I'm not going to have more than one, ever
    - If they're outside the ECS, maybe the app state can just own it. It would also be easier to fetch its transform to render meshes -->
<!-- - Setup keyboard events to move main camera around -->
<!-- - Actually figure out what 3d space this even is and get a non-trial-and-error rotation setup -->
<!-- - Limit vertical rotation to 90 degrees somehow -->
<!-- - Show camera and input parameters on debug widget -->
<!-- - Hide and lock mouse cursor whenever m1 is down -->
<!-- # Setup basic objects for grid and coordinate axes -->
<!-- # Setup a time variable and animate a material rotation -->
<!-- # Why is the cube rendering at the bottom left?
- Missing viewport -->

# I want to import a GLTF object
<!-- - Read files from a public folder into the wasm module -->
<!-- - Read gltf bin files into the module -->
- Parse gltf bin files
- Get object transform hierarchies working
- Get textures working
- Get simple PBR materials working 

# Move input stuff somewhere else
# Generated sphere mesh

# I think I'll need wasm-bindgen-futures at some point for something?
# I'm going to need some comprehensive logging to file functionality to help with debugging as I won't be able to step through at all...

# Cool sources
- https://github.com/bevyengine/bevy
- https://github.com/not-fl3/macroquad
- https://github.com/hecrj/coffee
- https://github.com/ggez/ggez
- https://github.com/mrDIMAS/rg3d
- https://github.com/PistonDevelopers/piston