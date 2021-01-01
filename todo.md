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
<!-- - Maybe create like a small slice of the components array, like a mini component manager and entity manager to store the imported gltf scene "prefab". Whenever want to spawn one we just copy it into the main one
    - This could be a "scene" as well
    - Sources (meshes/materials/textures) would be stored on the resource manager and shared
- Parse gltf bin files into webgl mesh data
  - Can create new entities and hierarchies and stuff now -->
<!-- - Add another local_normals material and a world_normals material to use for the duck for now -->
<!-- - I think I may need some coordinate conversion from GLTF, at least the up axes -->
<!-- - Implement splicing a scene into the current scene, to add the node hierarchy as it is in the GLTF file -->
<!-- - Need a parent node -->
<!-- - Actually parent one node to the other -->
<!-- - Construct unique identifiers for resources like meshes, materials and textures, because when parsing the nodes they'll be referred to directly -->
<!-- - Splice at transform x -> set tarnsform to scene root -->
<!-- - Flipped Y when going from blender to sv 
- Nested transforms are broken for the engine scene
    - They were both the same issue: I wasn't flipping the transforms from Y-up to Z-up -->
<!-- - Make material/mesh names unique in some way --> 
<!-- # Compile all engine materials up front -->
<!-- # Cleanup resource manager -->
<!-- # Honestly I may not even need the entity index inside Entity and always use just the uuid -->
<!-- # Less dumb way of storing/reading shaders -->
<!-- # Make it so that entity 0 is the "invalid entity". It's going to have some components, but who cares -->
<!-- # Sparse component arrays -->
<!-- - Likely wouldn't get any benefit from DOD if there are like 7 instances of the component in 2000 entities
- Hash map from entity to component
    - For now a "one for every entity" type of component and a "hashmap" type of component are a decent split, but later we could have another one that is also a contiguous array, but has an index switchboard  -->
<!-- - Have a component for entity metadata maybe
    - Don't need this yet, maybe later. I'll leave for when I have actual use cases -->
<!-- - Sparse light component -->
<!-- - Sparse UI component -->
<!-- # Lights and simple phong material -->
<!-- - Rendering system traverses sparse light components and sets the uniforms for the ones closest to the camera
    - Maybe do this only once every 10 frames or something like that? -->
<!-- - Multiple lights -->
<!-- - I think I can set the number of lights via a uniform?
    - Not in WebGL at least. I'll just carry on with a const num lights, and if performance becomes an issue I can text-replace different shader permutations for different number of lights, or still provide an uniform and break out of the loop whenever num_lights is reached -->
<!-- - Different light types
    - Directional light
    - Point light
- Multiple lights of different types -->
<!-- - Phong material
    - Need to have a material trait object and different materials, as the attributes/uniforms will be different -->
<!-- # Add normals to procedural geometry
- And UVs -->
<!-- # Procedural sphere -->
<!-- # Icosphere normals look weird -->
<!-- # Setup test scene for textures (plane, boxes and stuff) -->
<!-- # Load textures from public folder -->
<!-- # I want to import a GLTF object -->
<!-- - Get textures working
    - Import them from gltf
    - Import from raw bytes    
    - Allow cloning materials so that we can set custom parameters (like textures) for each
    - Allow modifying materials so that we can change uniforms at runtime (RefCell?)
    - Actually use textures -->
<!-- - Get simple PBR materials working
    - Fetch imported GLTF materials when parsing GLTF meshes
    - Have meshes use imported GLTF materials by identifier, like nodes use meshes -->
<!-- # Would be nice to have an "automatic" way of handling uniforms... there's a lot of repetition -->
<!-- # GLTF importer crate completely freezes if the file is like 8MB
- Maybe the WASM interface can't pass objects larger than X so that the GLTF importer starts reading garbage and never finishes?
- https://github.com/rustwasm/wasm-pack/issues/479
    - Actually the problem is the GLTF crate. I just bumped the opt-level -->
<!-- # Shader system refactor -->
<!-- - I need to support defines, includes and conditional compilation
    - Make Material struct detect when a texture is set, record a new define for it, and invalidate its program. When drawing, the rendering system would see an invalid program for a shader and provide it with the gl rendering context to recompile -->
<!-- - Test for more textures/scenes -->
<!-- - Provide tangents for the procedural spheres -->
<!-- - Need to upload inverse transpose for transforming normals -->
<!-- # Update to WebGL2 -->
<!-- - Better sooner than later as I need to update all shaders... -->
<!-- - Use VAOs -->
<!-- # String enum for uniform names -->
<!-- # I don't think I need this silly "get_component_index()" function, and can just use a trait const -->
<!-- # I kind of need entity names for debug -->
<!-- # At some point I put None when setting material for a mesh and it kind of looked like it was using the uv0 material?
- It was just vertex color -->
<!-- # Pass along an inverse transform to transform normals with -->
<!-- # Move camera `v` and `p` computation away from material. Probably all transform computation?
- Camera class somehow (probably not worth it being a component) -->
<!-- # Generated sphere mesh -->
<!-- # Update egui -->
<!-- - I think the main reason why I need the custom backend is to prevent it from clearing the buffer before drawing. And now so that I can get the context from it -->
<!-- - I think he fixed the column thing so I can probably make an aligned table for the debug widget -->
<!-- # UI refactor -->
<!-- - Remove UI "components", as that doesn't seem like the best idea -->
<!-- - Heavily refactor interface system into interface manager -->
<!-- - Put egui context directly on the app state. Each component can draw anything, like you're supposed to in imgui -->
<!-- - UI manager or something that will draw the scene-independent UI, like top menus, debug menus, notifications, etc. -->
<!-- - interface manager will raycast, find the hit entity and try to figure out what to display based on state (shift pressed, etc.) -->
<!-- - For debug we could even display arbitrary component data like mesh name, material uniforms, etc. -->
<!-- - Dragging a slider off the window is interpreted as a mouse down, and drops the selection -->
<!-- - interface system will also draw top-level UI -->
<!-- - Get colliders from GLTF too, and make bounding boxes for them if they don't have them -->
<!-- - It's a really bad idea to have a Mesh -> Collider -> Mesh Rc cycle... it can't ever be destroyed -->
<!-- - Need to set proper unique IDs on meshes now that the colliders rely on them -->
<!-- - Maybe reshuffle it a bit so that the actual intersection math is on raytracing utils -->
<!-- - Raycasting doesn't work on plane -->
<!-- - Disable raycasting when we're dragging with right click, as it chugs a little bit somehow -->
<!-- - We don't need to keep all of the data like vertex colors and normals... make a dedicated struct for it and move just the pos/indices arrays -->
<!-- - It has a noticeable effect on framerate... hopefully we can always just use bounding boxes -->
<!-- - Still no idea how to get UI to block raycasting
    - Looks like I can just traverse the Rects and Triangles emitted from all paint jobs -->
<!-- # Picking -->
<!-- - Need to test the ray intersection functions -->
<!-- - Maybe it would be neat to have a debug draw of bounding boxes -->
<!-- - Showing different UI for the picked thing
    - Expose some controls like material uniforms -->
<!-- # Switch to nalgebra, custom "decomposed" transform type using vec3 scale and an isometry -->
<!-- # Global transform stored as a Matrix4 -->
<!-- - Picking is broke -->
<!-- - egui rotation is expecting degrees but really they're rads -->
<!-- - It's again possible to get the weird behavior when pitch goes to 90 and -90 -->
<!-- - Rotation angle doesn't nicely match the FOV anymore -->
<!-- - Moving the sun didn't actually move the light position -->
<!-- # Maybe I can put the hack on the window and not the canvas? -->
<!-- # Maybe I can delay initializing the engine until run is called? Is that useful? -->
<!-- # Duality of being able to access the global state from the thread_local as well as passing it down the update chain -->
<!-- # Pass scene around instead of EC manager -->
<!-- # Async stuff
- I think the only way to get the async file prompt working is to spread the async virus all the way to the InterfaceManager so that it can defer from there and let js handle the request
    - We can use this for loading the assets, yes, but it doesn't work for the "UI callback" stuff as the winit event loop is not compatible with async yet
- The only way would be to make the entire thing async, otherwise we'd have to return from that function and somehow capture a static reference to the engine so that it can be filled with the fetched data whenever the future completes, which sounds like even more work
- We can't even move the entire thing into a web worker because WebWorkers running WebAssembly cannot receive events from JS (e.g. canvas, click events) (https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html)
- Think I can do this with a nasty hack by keeping track of the engine from javascript's side, and injecting the loaded stuff from there
    - This would even allow it to load assets on-demand. I'd just have a fire and forget "load this asset" js function called from rust that would inject the asset into the engine
    - Crap, when begin_loop is called the object is "moved into the function" and so I can't access it anymore even using JS hacks like putting the engine inside an html element -->
<!-- # Being able to do fetches and stuff from begin_loop allows us to load files from the manifest (even if on-demand loading doesn't work)
- The on-demand stuff may still work if we make it so that we can e.g. use the duck 'Mesh' object even if it's not loaded yet: It will dispatch the JS call to fetch and load it, and in the meantime use some default asset. On every draw it will check if available, and whenever it is, it can swap it for the new asset.
- I can probably just use build.rs to compile the manifest file automatically -->
<!-- # Note: Rotation3::from_matrix -->
<!-- # Try offsetting periapsis before transformation and compare -->
<!-- # Scene manager thing
- Dedicated window that can list all loaded_scenes
    - Show info like scene name, metadata, how many entities
    - Button to inject the scene at some location
    - Button to open the scene as a new scene
    - Button to delete scene -->
<!-- # Note: Use this pattern to combine responses from egui widgets:
let mut response = combo_box(ui, button_id, selected, menu_contents);
response |= ui.add(label);
-->
<!-- # Delayed asset loading
- Maybe I can use serde somehow? I mean, mesh will basically just serialize to the asset name...
- Right now we have to load all the assets we'll use up front. Later on when we have more assets and this becomes annoying, what we could do is just e.g. request -> foo.png texture -> dispatches call to fetch_bytes and immediately return a "temp texture" like source engine pink/black checkerboards -> Every time we draw, we check if our intended texture is ready. If its not, we use the checkerboard, but as soon as it's ready we start using it -->
<!-- # Testing
- npm command like 'npm run test', which builds the js in the same way, except some switch on index.js detects that it's a "test run" and instead of following the regular engine init path, it just calls into some other wasm rust functions that run the tests inside rust -->
<!-- - Rust has some testing stuff, but I'm not sure if I'll be able to use that.. I may need some regular function calls and stuff, which is not a catastrophe -->
<!-- # What is Vector3::identity() -->
<!-- # Solar system scene -->
<!-- - Implement new scene/close scene -->
<!-- - Implement loading orbital elements from CSV -->
<!-- - Good reference for coordinate system conversion: https://space.stackexchange.com/questions/19322/converting-orbital-elements-to-cartesian-state-vectors -->
<!-- - Other way around: https://space.stackexchange.com/questions/1904/how-to-programmatically-calculate-orbital-elements-using-position-velocity-vecto -->
<!-- - Checker calculator: http://orbitsimulator.com/formulas/OrbitalElements.html
    - Other one: http://www2.arnes.si/~gljsentvid10/ele2vec.html
    - Just use HORIZONS -->
<!-- - http://www.bogan.ca/orbits/kepler/orbteqtn.html
- Maybe put this stuff in a separate crate, as I don't think there is a good one for it -->
<!-- - Probably best to just precalculate these because the math is rough (has some numerical method stuff in there even), and we'd probably even benefit from having as many points as our orbit geometry, or else the body may drift in and out of it's orbit path -->
<!-- - Need to have this code inside the engine (as opposed to a python script) because I want to be able to just type in orbital elements and see an orbit -->
<!-- # Scene serialization with serde -->
<!-- - Derp, I'm overwriting the Rc<Mesh> with a new Rc<Mesh>, but the old one is still alive and being used by the components... -->
<!-- - GLTF-like, index based RON -->
<!-- - Have to load resources that were serialized with the scene -->
<!-- - Implement the fetch requests so that we actually receive the assets we requested
    - Have to also expand the "content_type" thing to also signal what to do with the asset when it arrives (e.g. inject it into the scene or not) -->
<!-- - Don't need to keep re-parsing the ephemerides every time, just do it once to spit out ephemerides and data
    - Store it in some kind of csv: One for planets, one for moons, asteroids, comets, etc. -->
<!-- - What to do with resources like meshes and textures? Export a binary blob?
    - Resources are largely going to be constant, so I can probably just export their name. When loading we preload all assets -->
<!-- - What about leveraging the fact that component arrays are mostly already packed? Maybe I can use serde and just dump the whole thing? -->
<!-- # Try setting up a simple orbit scene on rails/with physics -->
<!-- - Rails movement
- Crudest level: J2000 orbital elements
- Present for all bodies (including asteroids)
- https://ssd.jpl.nasa.gov/?sb_elem
- utils file for handling orbital stuff
<!-- - Class to describe orbital elements -->
<!-- - Function to try to parse NASA ephemerides output to search for it (already do it with regex in my old thing) -->
<!-- - Generate ellipse data from orbital elements struct
- main.js::addEllipse -->
<!-- - Generate keypoints for ellipse
- I came up with a hacky algorithm to accumulate more points on the peri/apoapsis, but I think the best thing to do is to just generate a circle and reuse it for all ellipses, just converting the orbital elements into a single Matrix transform
- This may force me to support non-uniform scaling though, but I think it's worth it: I'd likely have many ellipses in the buffer for no reason -->
<!-- - Engine interface function to load ephemerides files during execution
- Generate a small scene for it
- Inject scene into current like for GLTF scenes -->
<!-- - Function to convert to and from state vectors from orbital elements -->
<!-- - There are alternative orbital elements for coments and asteroids, as well as a two-line element... -->
<!-- - Functions to compute other stuff like period, periapsis, apoapsis, location of ascending/descending nodes -->
<!-- - Draw ellipses with webgl lines for now -->
<!-- - https://en.wikipedia.org/wiki/Simplified_perturbations_models -->
<!-- - SPICE: Only for planets, moons and the missions
- Would need to wrap the C library or use this: https://github.com/rjpower4/spice-sys, I don't think I can get it below 10 MB for the library alone, let alone the kernels with data
- Maybe do this in the future for the mission data... although I'd likely need to have data for all solar system bodies or else the spacecraft trajectories wouldn't match the target body -->
<!-- # Scene manager
- Likely use Serde
- Serialize the entity and component arrays in one go as byte buffers for now
    - Maybe ASCII too to help debugging -->
<!-- # I think I'll need wasm-bindgen-futures at some point for something?
- https://github.com/sotrh/wgpu-multiplatform/blob/41a46b01b6796b187bf051b7b0d68a7b0e4ab7f6/demo/src/lib.rs -->
<!-- # I'm going to need some comprehensive logging to file functionality to help with debugging as I won't be able to step through at all... -->
<!-- # Note: I should not change coordinates to use SSB (solar system barycenter) instead of heliocentric:
- Over years, the Sun slowly moves around by a few hundred thousand kilometers in response to the motion of the large outer planets Jupiter, Saturn, Uranus and Neptune. However, the inner planets keep fairly close to Kelperian orbits around the Sun wherever it happens to be at the time. (https://space.stackexchange.com/questions/24276/why-does-the-eccentricity-of-venuss-and-other-orbits-as-reported-by-horizons) -->
> - Can't type in egui slide fields
<!-- - Enter/Backspace stuff don't work -->
> - Typing WASD still moves
> - Maybe also handle wheel events and stuff while I'm at it?





# Get some planets orbiting
- Curate csv for only planets and main moons now since performance is junk
- Orbits not parented to eachother
<!-- - Weird aliasing/precision issue when drawing orbits as far away as jupiter
- Weird issue where if we go far enough from origin everything disappears and we get NaN on camera position, even though clip space is much farther and 
    - Flickering things was the camera near plane being too near and far being too far. I patched it with better numbers but later we'll want logarithmic depth buffers like in threejs -->
- Some controls of orbital time (e.g. JDN per second)
    - Separate to physics simulation time?
- Metadata dictionary HashMap component where orbital elements can be placed. If available we build and concatenate a transform for it on import
    - Also store other stuff like mass, magnitude, rotation info    

# Actually what I really need is the conversion in the other direction: State vector -> osculating orbital elements. I can run this one on free bodies to do orbit prediction, maybe?
- https://space.stackexchange.com/questions/24276/why-does-the-eccentricity-of-venuss-and-other-orbits-as-reported-by-horizons

# Logarithmic depth buffer

# Crazy slow (11 fps with all planets and moons loaded and it's not even updating positions yet)
- Maybe because I set/unset GL state every time? Would mean it's unrelated to geometry tessellation level
- Use profiling crates (like tracing https://crates.io/crates/tracing)

# Add tests for raycast intersections

# The app will never know the full path of the GLB file when injecting, only if we keep track of the original URL ourselves (won't work for runtime uploads though...)
- I have to revive the manifest thing and basically make sure all glb/texture assets have unique names, because if we hit a scene that was saved with assets that were uploaded at runtime all it will say is "albedo.png" and we won't know its folder

# Store camera/app state to local storage so that it reloads facing the same location

# I can tweak the egui visuals a bit. Check out the demo app menus on the left

# What about that transform feedback thing? Maybe I can use that for line drawing or?
# Better line drawing
- How did my JS app do it? The lines there seemed fine...
- Line "tessellation" into triangles for thick lines
    - Geometry shaders? Not in WebGL2...
    - Will probably have to do a double-sided quad cross like old school foliage stuff
        - Would look like crap up close.. maybe if I added an extra uniform to vertex shaders to have them shrink when you approach
# Free body movement
- Gravity, force application
- Orbital trajectory prediction -> line drawing
# Put orbital mechanics stuff in another crate once it's big enough
# I kind of really need to use f64 everywhere
- Argh WebGL2 doesn't really support it in the shaders, so there's likely not much point
    - https://prideout.net/emulating-double-precision
# Can probably fix the initial "Request for pointer lock was denied because the document is not focused." by... just focusing the document on right click
- It seems fine on Chrome so I think I'll jus tignore this for now
# Can probably fix the framerate thing by keeping a rolling sum of N frames and their times
- Demo app has an x seconds per frame thing on the left which is just what I need
# I think I'm doing some of the stuff in https://webgl2fundamentals.org/webgl/lessons/webgl-anti-patterns.html
<!-- # Remove pub use sub::* when I don't mean to, usually pub mod does what I want and keeps the namespace on the import path, which is neater -->
# Cubemap texture for the skybox
# Some way of specifying parameters for procedural meshes, or hashing the parameters, etc.
# Find a better way of handling component updates when sorting after reparenting
# Annoying bug where if you drag while moving the += movement_x() stuff will add to an invalid mouse_x as it never run, making it snap
# GLTF importer crate can't handle jpg images as it tries spawning a thread to decode it
# I think it should be possible to store the shaders in the public folder and tweak the automatic reloading stuff to allow hot-reloading of shaders
- WasmPackPlugin has a "watchDirectories" member that I can use to exclude the public folder
# We always try using vertex color, even if it contains garbage in it.. need to be able to set some additional defines like HAS_VERTEXCOLOR, HAS_TANGENTS, etc.
# Maybe use https://crates.io/crates/calloop for the event system instead

# Docs link:
- file:///E:/Rust/system_viewer/target/wasm32-unknown-unknown/doc/system_viewer/index.html

# Cool sources
- https://github.com/bevyengine/bevy
- https://github.com/not-fl3/macroquad
- https://github.com/hecrj/coffee
- https://github.com/ggez/ggez
- https://github.com/mrDIMAS/rg3d
- https://github.com/PistonDevelopers/piston
- https://github.com/wtvr-engine/wtvr3d/blob/8fcbb69104e0e204a6191723589013ddbd66629b/src/renderer/uniform.rs
- https://github.com/bridger-herman/wre/blob/6663afc6a5de05afe41d34f09422a7afcacb295c/src/frame_buffer.rs
- https://github.com/bridger-herman/wre/blob/6663afc6a5de05afe41d34f09422a7afcacb295c/resources/shaders/phong_forward.frag

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

# Orbital mechanics:
- Orbital mechanics equation cheat sheet even for non-elliptical orbits: http://www.bogan.ca/orbits/kepler/orbteqtn.html
- https://ssd.jpl.nasa.gov/horizons.cgi
- Great space SE threads:
    - https://space.stackexchange.com/questions/19322/converting-orbital-elements-to-cartesian-state-vectors
        - https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
    - https://space.stackexchange.com/questions/1904/how-to-programmatically-calculate-orbital-elements-using-position-velocity-vecto
    - https://space.stackexchange.com/questions/24276/why-does-the-eccentricity-of-venuss-and-other-orbits-as-reported-by-horizons
        - On this one they recommend using heliocentric if I intend to use osculating orbital elements, as they tend to be more consistent
    - https://space.stackexchange.com/questions/25218/why-is-the-sidereal-period-of-the-earth-362-392667-days
    - https://space.stackexchange.com/questions/23408/how-to-calculate-the-planets-and-moons-beyond-newtonss-gravitational-force/23409#23409
    - https://space.stackexchange.com/questions/15364/pythagorean-three-body-problem-need-some-points-from-an-accurate-solution-fo
    - https://space.stackexchange.com/questions/23535/compatibility-of-osculating-elements-and-cartesian-vectors-given-by-jpl-horizons/23536#23536
    - https://space.stackexchange.com/questions/22915/calculating-the-planets-and-moons-based-on-newtonss-gravitational-force/22960#22960
        - Crap: Also, if you calculate their position relative to the Sun yo For Io, you have semi-major axis of 0.003 AU orbital radius, at distance of 5.2 AU from the Sun. This works poorly with floating point values as the fixed, large offset from center of the system of coordinates prevents them to increase precision by changing the mantisse and changes occur at the far tail of the base with a lot of precision lost behind its tail. 
        - Also I would hold off on Jupiter's moons until you add higher order gravitational multiples (e.g. J2) due to oblateness.
        - After that, there are two big corrections to simple point-mass to point-mass Newtonian gravity that I can think of; 1. General Relativity (GR), and 2. higher-order gravitational multipoles (e.g. J2 and beyond) and tidal forces. Mercury will show a big improvement with GR, and the Earth-Moon and moons of giant planets will show improvement with non-point gravity. 
        - Aside from numerical issues, "With Sun as centre" may be part of your problem. Get all the data from Horizons relative to the Solar System Barycenter, not the Sun, which moves relative to the barycenter. That barycenter is an inertial frame of reference, whereas the center of the Sun is not. Also make sure that you are putting in the initial position and velocity of the Sun and letting it move, if you aren't already.
    - https://scicomp.stackexchange.com/questions/29149/what-does-symplectic-mean-in-reference-to-numerical-integrators-and-does-scip
    - https://space.stackexchange.com/questions/22948/where-to-find-the-best-values-for-standard-gravitational-parameters-of-solar-sys