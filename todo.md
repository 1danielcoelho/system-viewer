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
<!-- # Q and E to go up or down -->
<!-- - Setup physics system to be able to set an object rotating -->
<!-- # Show some statistics on the debug thing -->
<!-- - Framerate counter -->
<!-- - Control simulation speed -->
<!-- - Read files from a public folder into the wasm module -->
<!-- - Read gltf bin files into the module -->
<!-- - Generational entity indices
    - Index, generation and uuid
    - uuid is monotonically incremented and never changes for an entity, even if reordered
    - block direct access to component arrays when fetching other entities
        - It's fine for systems though, they'd still go through them continuously
        - Check target index, if generations don't match search for uuid
            - Map from uuid to current index
    - When an entity is dropped mark it as dead, forget its uuid so that search fails
    - Have to iterate through entities when executing a system, to know if the owner entity is live -->
<!-- - Entity references should just be UUIDs. What's the point of using generational lookup for them? -->
<!-- - Keep some state on ent man about resorting entities after a reparent?
    - Remember to update free indices and uuid to index
    - entman -> compman event? Maybe have the event trigger a variable in comp_man, and have it update them before running for a frame -->
<!-- - Delete entity should remove it from its parent and delete it's children as well -->
<!-- - Move canvas event stuff into engine_interface -->
<!-- - Scalings aren't working, I think I messed up the transforms -->
<!-- - Get object transform hierarchies working
    - Keep world_transform and local_transform on components
        - Maybe keep local_transform inside an optional? I guess it makes no difference
    - When reparenting a transform to another, sort entities so that parents come before children
    - Separate system to propagate transforms that runs after physics system updates
        - This may be a problem later when computing collision and using child BB but let's ignore it for now
        - Physics system should completely ignore component if it has a parent -->
<!-- - Rendering system should read off world_transform -->
<!-- - How to reconcyle physics system with transform hierarchies?
    - Constraints? Probably way too much for now. Likely just skip linear movement if child
    - When computing the physics stuff for the parent, we'd have to factor in the mass/momenta of the children too, then rip cache coherence
    - I think for now children should be completely frozen wrt parent. Later on we can add some fancy pass to propagate stuff upward if needed or something like that
    - Will probably have to make sure that parents always come before children in the entity array
    - Does entity order even matter if entities can't have moving sub-parts?
        - It should be simple and quick to make sure parents come first
        - Maybe use a depth index on the transform component?
    - I may need total transform for other systems at some point, so they may need to be stored inside the transform component, and propagated to children on physics component that runs after it -->
<!-- - Disable physics component for sleeping stuff, like the grid or axes entities -->
<!-- - I don't resize the components array when doing new_entity... if I use the new entity to swap with another, we may lose our components -->
<!-- - I don't think I need the generational entity thing if I'm using uuids... -->
<!-- - Tons of indirection when scanning through transform components -->
<!-- # Move input stuff somewhere else -->

# I want to import a GLTF object
<!-- - Maybe create like a small slice of the components array, like a mini component manager and entity manager to store the imported gltf scene "prefab". Whenever want to spawn one we just copy it into the main one
    - This could be a "scene" as well
    - Sources (meshes/materials/textures) would be stored on the resource manager and shared
- Parse gltf bin files into webgl mesh data
  - Can create new entities and hierarchies and stuff now -->
<!-- - Add another local_normals material and a world_normals material to use for the duck for now -->
<!-- - I think I may need some coordinate conversion from GLTF, at least the up axes -->
- Implement splicing a scene into the current scene, to add the node hierarchy as it is in the GLTF file
    - Need a parent node
    <!-- - Construct unique identifiers for resources like meshes, materials and textures, because when parsing the nodes they'll be referred to directly -->
- Get simple PBR materials working
    <!-- - Make material/mesh names unique in some way -->    
    - Fetch imported GLTF materials when parsing GLTF meshes
    - Have meshes use imported GLTF materials by identifier, like nodes use meshes
- Get textures working

# Scene manager
- Likely use Serde
- Serialize the entity and component arrays in one go as byte buffers for now
    - Maybe ASCII too to help debugging
# Testing
- npm command like 'npm run test', which builds the js in the same way, except some switch on index.js detects that it's a "test run" and instead of following the regular engine init path, it just calls into some other wasm rust functions that run the tests inside rust
- Rust has some testing stuff, but I'm not sure if I'll be able to use that.. I may need some regular function calls and stuff, which is not a catastrophe
# Update egui
# Honestly I may not even need the entity index inside Entity and always use just the uuid
# Have a component for entity metadata maybe
# Sparse component arrays?
- Likely wouldn't get any benefit from DOD if there are like 7 instances of the component in 2000 entities
- Hash map from entity to component
# Move camera `v` and `p` computation away from material. Probably all transform computation?
- Camera class somehow (probably not worth it being a component)
# Generated sphere mesh
# Annoying bug where if you drag while moving the += movement_x() stuff will add to an invalid mouse_x as it never run, making it snap
# I think I'll need wasm-bindgen-futures at some point for something?
- https://github.com/sotrh/wgpu-multiplatform/blob/41a46b01b6796b187bf051b7b0d68a7b0e4ab7f6/demo/src/lib.rs
# I'm going to need some comprehensive logging to file functionality to help with debugging as I won't be able to step through at all...

# Docs link:
- file:///E:/Rust/system_viewer/target/wasm32-unknown-unknown/doc/system_viewer/index.html

# Cool sources
- https://github.com/bevyengine/bevy
- https://github.com/not-fl3/macroquad
- https://github.com/hecrj/coffee
- https://github.com/ggez/ggez
- https://github.com/mrDIMAS/rg3d
- https://github.com/PistonDevelopers/piston

# Physics
- https://www.toptal.com/game/video-game-physics-part-i-an-introduction-to-rigid-body-dynamics
- https://gafferongames.com/post/physics_in_3d/
- https://github.com/DanielChappuis/reactphysics3d
- https://gafferongames.com/post/integration_basics/
  - It looks like semi-implicit Euler integration should be fine for now, and should be pretty easy to implement. Later on I can switch to RK4 if I need to
- https://github.com/dimforge/nphysics/blob/fcb91b27dd5cf8a5ce9684e3b99e1788a39d3619/src/object/rigid_body.rs#L599
  - Gyroscopic forces sample
- https://github.com/idmillington/cyclone-physics/blob/fd0cf4956fd83ebf9e2e75421dfbf9f5cdac49fa/src/body.cpp#L154
  - Rigid body integration sample
- Use "simulation islands" to sleep large areas at a time
- Concept of using "energy" to detect when an object should sleep (e.g. too little kinetic energy)
  - Use an enum to store the current status of the object: Asleep or active (with x energy)
- Decouple simulation timestep from the actual passage of time so that it can be controlled
- https://github.com/RandyGaul/qu3e/blob/a9dc0f37f58ccf1c65d74503deeb2bdfe0713ee0/src/dynamics/q3Island.cpp#L38
  - Another sample of semi-implicit Euler
