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
> - Get some planets orbiting
> - Curate csv for only planets and main moons now since performance is junk
> - Orbits not parented to eachother
    > - My previous JS code to calculate the position just seems flat out wrong to me
    > - Also why was I baking the eccentric anomaly and then doing a bunch of extra (wrong) calculations after it? If I'm baking anything I might as well just bake the final positions and interpolate them. I could even use the transformed ellipse vertices so that it always follows the trajectory nicely as well
    > - Current problem is that the sun barycenter doesn't have a mass (or something) so that no E samples get baked. Maybe I need to start using orbital period as a parameter
    >     - Also would solve other annoyances like: What is the "mass" that I should put for the reference mass for a moon of jupiter? Jupiter's mass or the system's mass? etc.
<!-- - Weird aliasing/precision issue when drawing orbits as far away as jupiter
- Weird issue where if we go far enough from origin everything disappears and we get NaN on camera position, even though clip space is much farther and 
    - Flickering things was the camera near plane being too near and far being too far. I patched it with better numbers but later we'll want logarithmic depth buffers like in threejs -->
> - Some controls of orbital time (e.g. JDN per second)
>     - Separate to physics simulation time?
> - Metadata dictionary HashMap component where orbital elements can be placed. If available we build and concatenate a transform for it on import
>     - Also store other stuff like mass, magnitude, rotation info    
> - Crazy slow (11 fps with all planets and moons loaded)
> - Maybe because I set/unset GL state every time? Would mean it's unrelated to geometry tessellation level
> - Use profiling crates (like tracing https://crates.io/crates/tracing)
>     - https://github.com/storyscript/tracing-wasm
>     - Chrome's performance tab already can fetch function names and do waterfall charts and stuff
> - Christ: For the full scene the OrbitalSystem takes 31.8ms/frame, the transform update system takes 11.95ms/frame and the rendering system 45.31ms/frame
    > - OrbitalSystem takes 4ms/frame now, which is almost passable 
    > - A substantial ammount of this is nalgebra stuff, although I don't think I could do better: I need to just use it less
    > - Just change to opt-level 2 for now: It compiles about as fast and performance is miles better (pinned to 60fps, 5ms / frame)
> - Body positioning on top of orbit is not exact: They're always slightly closer to center
> - Set camera reference frame to a moving body
> - It should be possible to click on a body to lock the camera to it
> - Allow clicking on a "system barycenter" somehow
    > - Not needed for now and users may not even know barycenters are meant to be entities
> - Something is wrong: The moon doesn't fully orbit, it only moves like 1/5th of a rotation and snaps back    
> - I think raycasting doesn't work if the camera has a reference
> - Spacebar to pause?
> - I'm not really sure if the planets are the right size, need to measure it somehow
> - Store camera/app state to local storage so that it reloads facing the same location
> - Movement speed needs to be scaled when resetting camera reference
> - Don't want to waste time on this for now, because I'll probably change this anyway
> - I need some kind of window to list of entities
> - Need to have a "move to entity" button
> - Change how camera reference works to maybe just append the translation of the reference or something, because it's very annoying to have the scale of the target body affect the camera transform/precision issues
> - This would also fix the movement scaling issue when changing reference
> - It could be useful to have camera near/far adjust like this though.... although we likely need a more robust system as I always want near/far/speed to scale down when we're near a body, whether we're using it as a reference or not
> - Floating point precision issues
> - Pluto wiggles severely. I think f64 is way more than fine for the orbital calculations, but it has to get cut down to f32 for WebGL
> - https://prideout.net/emulating-double-precision
> - https://blog.cyclemap.link/2011-06-09-glsl-part2-emu/ 
> - https://programming.vip/docs/5ec76758067e9.html
> - https://github.com/visgl/deck.gl/blob/master/docs/developer-guide/64-bits.md
> - I can't even move if the move around pluto's orbit if the move speed is below some amount 
> - GL_ARB_gpu_shader_fp64 extension?
> - Fixed by just passing WVP instead of VP and W into the shaders and combining, because this way the object positions are (almost) always wrt to the camera
>     - Orbits will remain a bit wiggly though, because e.g. pluto's orbit is still centered at the sun, but we still look at the line from pluto
>         - We could fix this by just moving the length units a few exponents up: The orbits will always be the issue, because the planets/bodies don't look so bad as we only see them wrt. the camera
>         - It doesn't look prohibitively bad right now, and I'm not even sure how I'll draw the orbits in the end, so I'll just ignore this for now
    > - Lighting calculations will be wiggly (because I use light/frag coords in world space), unless we do all lighting in camera space (which we might)
> - Component storage refactor
> - Anymap
>     - Also difficult to split borrows (e.g. some system needs read access to physics components and write access to transform components)
>         - May be possible by putting RefCell<ComponentStorage<T>> inside the anymap
>         - Kind of sucks that it would pay the cost of runtime borrow check for every operation just for this usage
>     - Serialization
>         - No support from AnyMap, so I'd need some auxiliary data structure
>         - Maybe keep a tally of all entries that are registered/unregistered in the AnyMap, like a Vec for "ComponentIndices that were registered as vec". In some unlucky function somewhere I'd go through this tally and somehow try to query AnyMap for a type based on a value?
> - I could also maybe store a Vec<Box<dyn ComponentStorage>> or something like that?
>     - May be easier to serialize since I can easily iterate over them?
>         - Maybe I could make ComponentStorage : Serialize + Deserialize and have each struct implement it? Not sure if dynamic dispatch would work like that
>     - Could use a HashMap<ComponentIndex, usize> to know the index of each storage?
>     - Naturally prevents multiple storage types for the same component type
>     - Naturally restricts all storages through a common trait, which is neat
>     - May allow me to split references by using split at mut on the Vec of storages?
>         - I can't return the spit borrows though so I'd need some crazy refactoring of systems to work based on closures or something, and call the closures within a Scene function where I can split borrows
>     - Would be hard to type-erase the component types (have to use trait base classes and so on)
>     - Would likely be the most indirect method
> I want to be able to more easily control the simulation scale (from day/s, seconds/s, years/s)
> Component storage refactor
> - Serialization of sparse storage is just duplicating the entity to index Rc for each component
> - N-body simulation
> - Separate component than the orbital component. Maybe even something separate to the physics component entirely
> - Looking at the space stackexchange it seems imperative to compute all forces before all positions/velocities update
> - For solar system n-body it seems critical to also use the solar system barycenter as the origin and let the sun move, in contrast with using the sun as the origin for rails simulation (due to how the planet orbits are closer to constant ellipses wrt. the sun)
> - Introduce packed component storage with index switchboards to speed things up
> - Parse mass more carefully, since the exponent seems to always change
    > - Lots of bodies don't have mass listed. I think we'll have to estimate it based on orbital period and reference body
> - Asteroid data
> - Parse data from horizons and compile it in big, *standardized* json files
    > - Use this to complement it, if they don't match: https://ssd.jpl.nasa.gov/?sat_phys_par    
    > - One JSON file for all data we have on each class: One big file for all planets, one for all jupiter satellites, one for all asteroids, etc
    > - Store original epoch of osculating elements, and have an easy way to roll it forward and back before computing state vectors
    >     - I don't think this is a great idea.. it would require some annoying computation like doing the same for each parent and then concatenating the transforms, and the result would be grossly inaccurate as those were just osculating elements anyway. I'm better off just having state vectors at J2000 for everything
    >         - For asteroids/small bodies I'll probably have to estimate it anyway, but at least they're always just wrt. sun
> - Adapt python script to convert ephemerides orbital elements files into json data
> - Adapt python script to read downloaded state vectors and add them to the read json data
> - Manually copy rotation axis data from the downloaded PDF into some text file 
> - Python script to convert rotation axis data and load it into json files
> - I stopped at trying to make a regex that matches one entry, whether that is for orbital elements or state vectors
> ``` JSON
> {
>   "0": {
>     "name": "Sun",
>     "type": "star", // star, planet, satellite, asteroid, comet, artificial, barycenter
>     "meta": {}, // data sources, future proofing, etc.
>     "mass": 2, // kg
>     "radius": 2, // Mm
>     "albedo": 2, // abs
>     "magnitude": 2, // abs
>     "rotation_period": 2, // days (86400s)
>     "rotation_axis": [0.1, 0.8, 0.2], // J2000 ecliptic rectangular right-handed normalized
>     "spec_smassii": "Sk", // Spectral class
>     "spec_tholen": "B", // Spectral class
>     "osc_elements": [
>       {
>         "ref_id": "0", // reference body id
>         "epoch": 2455400.0, // JDN
>         "a": 2, // Semi-major axis, Mm
>         "e": 2, // Eccentricity, abs
>         "i": 2, // Inclination, rad
>         "O": 2, // Longitude of ascending node, rad
>         "w": 2, // Argument of periapsis, rad
>         "M": 2, // Mean anomaly, rad
>         "p": 2, // Sidereal orbital period, days (86400s)
>       },
>     ],
>     "state_vectors_ssb": [
>       [2455400.0, 20, 20, 20, 10, 10, 10], // JDN, Mm, Mm, Mm, Mm/s, Mm/s, Mm/s
>       [2455401.0, 30, 30, 30, 10, 10, 10],
>     ],    
>   }
> }
> ```
> - Get asteroid and comet entries into the database
>     - Elements can come from the downloaded existing files, but we need state vectors
>         - They're all heliocentric, so just roll their osc_elements to J2000 and convert (ugh)
>         - Work on testing elements->cartesian conversions in python
>         - It may still be slightly off because Horizons uses some other osculating element solutions
    > - Radius estimation from albedo/magnitude
    > - Radius/mass estimation
    > - https://space.stackexchange.com/questions/36/how-can-i-derive-an-asteroid-mass-size-estimate-from-jpl-parameters
    > - https://space.stackexchange.com/questions/2882/method-to-estimate-asteroid-density-based-on-spectral-type
    > - Atm we got mass for 23% of all bodies, which should be more than enough
    > - Comets don't just have one magnitude, they have many
        > - Comet mass estimation
        >     - I think this is impossible given that JPL has *no* mass values for any comet
> - Bugs
> - Maybe have alt + scrollwheel be zoom
> - If you are above an object and look down, the label goes on the wrong location
> - Labels disappear when too close to near plane
> - I kind of want these labels to always be on the screen, even if the object is out (like an arrow indicator in games)
> - Offset of label to body is incorrect
> - Labels seem like 1 frame behind somehow?
> - Labels projection math doesn't work if body is reference (maybe using/not using the camera reference would fix it?)
> - Labels freak out if they're behind and we have a body as reference, or even when having no reference at all
> - Clicking the UI doesn't always consume the click, so that e.g. it may clear selection when clicking on cog
> - Clear out confusion of what should happen when we click/reference a body
    > - Label should expand to show distance and body info (just distance for now)
    > - Selecting and tracking should be separate things: Selecting a body will show its name and two buttons: Go to and track
    > - Once tracking, left click and drag will orbit, mouse wheel will zoom
        > - I'll not worry about mouse wheel zoom for now because that's the speed control.... I'm not sure what I'll do about that yet but its easy to zoom by just moving around with the keyboard. Maybe I'll even expose FOV better for actual zooming instead
        > - Probably better to do it with Alt+Left click and drag, or else I'll have to watch out for what happens if we release drag on top of a clickable, etc.
            > - It seems like the modifiers only capture if the canvas has focus, and Alt removes focus...
        > - I thought it would be a problem that I'll have to update position based on reference_transform before the transform update system would get to run, but it is not a problem at all: The camera's position to its reference only ever changes from input data, so it will be fine. I just do everything wrt. reference transform and later on that gets updated instead
    > - You should be able to select other things when tracking, but you'll remain tracking the same object regardless of your selection
    > - The label on the top toolbar should change to yellow when tracking to highlight what's going on
    > - Maybe change it to red if we try orbiting without a tracked target
    > - Change tracking remove icon to X, as it kind of looks like you'll delete the body if you click on it
> - It doesn't look like it orbits about the actual target...
> - Moving the cursor over UI during drag cancels the drag
    > - Maybe the simplest thing is to not pass the updated mouse position to egui during mouse capture
> - Click and drag should also capture the cursor
> - Being able to independently change the target while orbiting doesn't work so great with locked pitch. I'll just abandon this for now and always snap target to 0 when orbiting
> - Always look directly at target when starting a track
> - Big old glitch when clicking a body in the scene hierarchy
> - Initial splash screen with picker for different scenes
    > - Scenes can be baked in, and load a subset of objects from json each
    > - Load data from new json databases, J2000 N-body simulation
        > - For some reason planets don't have masses?
        > - I think they're spawning with wrong positions/velocities, I may need to make a debug GUI thing for the physics component
        > - Fix planets instantly just falling into the sun
            > - Initial speeds and velocities seem reasonable, gravity is just too strong somehow?
        > - Create rust analogue of SceneDescription
        > - Parse scene_list.ron 
        > - Show SceneDescriptions on ui
        > - Parse json body databases on demand
        > - Single ron file with list of scenes. Each item would have a scene name, description, number of bodies and a list of body_ids and dbs to check. When clicking it, it would parse all required dbs and fetch the bodies required
> - Simple menu on top right with time, fps, body count, tracked object (+ untrack button) and cog menu
    > - Controls to set simulation speed right there
    > - Object list (can click to go to and track)
    > - About
    > - Option to go back to splash screen
    > - Fix scenes not loading meshes
    >     - This is because we were relying on the test scene to load the meshes and stuff
> - Controls and manipulation tweaks    
    > - Show object name on hover, open details on click    
    >    - Track object on click
    > - Switch camera to orbit mode when tracking an object, mouse wheel to zoom
        > - Have the "go to thing" system be more aware of the thing's size. It's impossible to find phobos and deimos for example, because they're too small    
>- Refactor body database
    > - Keep existing databases for body data (mass, size, color, albedo, etc.)
    > - Additional "state_vectors.json" file for known, reference state vectors (single file, each id has n state vectors, each vector contains epoch, sorted by epoch)
    > - Additional "osc_elements.json" for reference osculating elements (single file, each id has n elements, each element contains epoch and reference, sorted by epoch)
    > - Scenes in ron files
        > - Separate folder for scenes
        > - One scene per ron file (instead of scene_list.ron)
        > - Each scene can contain a custom state_vector/osc_element for a body. If it's not available, it will be fetched on the databases and estimated back/forward from reference data
        > - Allow leaving camera pos/target/up as None and just use reference, so that it does a GoTo when loading the scene and figures it out by itself
        > - Actually implement setting initial time and reference from scene desc
            > - Set initial time
            > - Metadata component
                > - Move entity names onto it
                    > - Gave up on this for now because names are fetched very frequently by the UI, and I don't want to go through get_component every time
                    > - Will have to have some consumer code start using entity ids for display though
                > - Store body id and description on it
                > - Linear search for body id when setting reference (I don't see it being that useful yet)
                > - Fix 1 frame flicker when setting camera reference at first
            > - How to handle the choice between osc_element and state_vector?
                > - This is worthless now as I don't want the ellipses back for the MVP
            >- Crash when closing a scene (should reset to empty)
> - Scene handling
    > - I think I should put the scene description inside the created Scene?
    > - Set time, camera position and reference according to scene description
    > - Reset seleciton when opening scene
    > - Option to reset to start
        > - Just open the current scene again
        > - Change open button to 'reset'
    > - Unload scene when switching away from it
>- Had to rotate each emitted one (from the rightmost 6 separate image options on https://360toolkit.co/)
    >- Back: Left
    >- Right: Left Left
    >- Bottom: Right
    >- Top: Left
    >- Front: Left
> - Just fork egui instead of having a custom gui_backend
>- Make sure its legally OK to do that though 
>- Doesn't work very well, because I really need egui_web, which is a subcrate within the egui repo, and I can't get cargo to use subfolder paths for git dependencies
> - Update egui
    > - UI labels are not translucent anymore
    > - Clicking on Track buttons doesn't work 
    > - No way of preventing movement when typing because the has_kb_focus member is private now
      >  - I opened a bug issue on egui's repo, so let's wait for that. If it's intentional we'll have to work around it somehow
    > - Maybe make the clear tracking style a little bit more visible
    > - Maybe add a colored "Tracking" button to the selection tooltip
    > - Can't click on main toolbar stop tracking button if the selected widget is opened for some reason
    > - Fix scroll wheel not scrolling scroll panels
    > - Still get all the old bugs with the flickering labels
    > - Can't collapse the popup windows because any click closes them
    > - Have a whole lot of duplicated/unused code in gui_backend
    > - Hover labels show when right-click dragging
> - Do something for movement speed, it's almost always unexpected
    > - I also have FOV basically unused...
> - Restore F to track something
> - Hover labels shouldn't show when right click and dragging
> - Hover label shouldn't show on top of selected label or other windows
    > - Selected label is just another window, so the draw order depends on which was active last... not sure if this is actually a bug or not, as its very easy to work around and it may be what you want
    > - The hover label prevents the scene browser from getting a hovered state, because it blocks raycasts
> - Should be able to click through the hover label (it blocks all mouse clicks)
> - Window that shows all the controls (ultra extra nice to have: rebind them)
> Window showing controls
> Window showing settings (show starmap or not, etc.)
> Starfield skybox
> - Expand on framerate limiter to allow frame stepping
    > - I'll do this only if I need to, for whatever reason. Don't really need this
> - It doesn't save the focused entity or the last opened scene
> - Also save egui state somehow
> - Improve visuals a bit
    > - Add some default color/texture to the body schema. I think I had colors for all planets from before?
    > - Passable materials/colors for bodies
    > - https://www.solarsystemscope.com/textures/
    > - https://svs.gsfc.nasa.gov/4851
    > - Used https://360toolkit.co/ to convert to cubemap
    > - Find a decent skybox and align it roughly
> - Fix normal and spec/roughness maps for Earth
>- Fix weird shoreline artifacts on earth metallicroughness
    > - Fixup gltf importer to allow importing GLTF scenes as simple meshes and materials, instead of making entire scenes with them
    > - Also can probably get rid of all the "scene injecting" stuff and provisioning stuff
    >- Actually it is completely right: It just so happen that you can't see the specular highlight at all for roughness = 0, because it's a perfect mirror, which is why the oceans looked black. I just edited the levels of the MR map, and it looks ok-ish now
>- OrientationTest and all multi-node scenes are messed up, because there's something wrong with the transform baking thing
>- DamagedHelmet looks black?
    >- UVs look messed up
    >- Looking at the UV sets in blender it looks like it expects some type of UV repeat mode being set, and I just do whatever at the moment
    >- I think it doesn't have any tangents, and it was relying on that automatic tangent generation from the reference shader, that uses derivatives and so on
    >- I have to detect when a model has tangents or not, and then set a define. On the shader, based on that define I enable/disable the support for generating tangents/normals via dFdx
>- MetalRoughSpheresNoTextures are so small that upscaling the geometry later leads to precision issues
>- Apparently dropping the inverse transpose and just using the transform on the normals seems to fix some of the normal issues, but I don't know why
    >- I'm already doing the inverse transpose when sending the normals to the shaders for the gltf_metal_rough material
    >- The inv trans compensation when concatenating is *required* if the nodes have non-uniform scaling, but for some reason it flips normals sometimes
    >- Apparently you *have* to remove the translation before doing inv_trans, because transform_vector won't magically ignore the inv transpose of your translation when transforming a vector
> - NormalTangentTest is messed up (likely the same as DamagedHelmet and shoreline issue)
    > - It's fine. It's not showing anything because I don't have an environment map
>- NormalTangentMirrorTest is messed up
    >- It's not: The readme just does a bad job of describing what it should look like. Comparing against the reference viewer it looks great
> - Also some wrong stuff with blend mode and texture settings, but not sure if it's worth doing anything about those at this time
>- There is something wrong happening when roughness is exactly 0. It just flips to rough again.. this is likely the shoreline thing
>- Pretty sure I'm not using the "provisioning" stuff I used to do, but I think it's used for the old GLTF loading path?
>- Tangents don't look smoothed on the uv sphere
    >- What about meshes that are delay-loaded?
>- What about meshes that are delay-loaded?
    >- Implemented a lazy 'compatible prim hash' system to recompile materials during rendering, if necessary. Likely not the most efficient solution but its the safest and most straightforward for now while I'm still changing things about materials
    >- Another problem is sort of preventing this from being a problem:
        >- When we encounter a mesh we need to delay-load, the temp mesh doesn't have any primitives, so we never create any material slots. This means that we're using just the default material. Whenever the gltf mesh does arrive, it will set its own materials which will be used instead
        >- Fixed by just using the materials assigned to the body in the database file as overrides. For now we only have one material per body, but later we could have more. Those are available immediately and would always become the material overrides
    >- It would be neat if I could have events, so that once a mesh has finished loading all the interested meshcomponents can respond and override their materials/update defines
        >- Still wouldn't fix it though because I'd have to get the body material after it has finished loading... 
        >- Also, even if I had Mesh being observable, I'd have to modify the swap code as I wouldn't be able to swap it for the new one: I'd need to manually swap primitives and name
        >- Overkill for this
>- Syntax to express batch objects on scene files (e.g. all major bodies with default vectors)
>- Some bodies don't show names on scene hierarchy?
>- Rename scene hierarchy to scene browser
>- I need some type of search on scene browser...
>- Need to be able to see bodies from afar somehow... 
  >- What if I do another render pass where I always draw a single pixel for each body?
  >- Single "points" mesh that is drawn with vertex color for each 
      >- Profile it a bit because I think I'm doing a lot of stuff I don't need to, like transforming a point when calculating last drawn position..
      >- Support vertex colors based on body type            
      >- Maybe I can reuse these last_drawn_position things for the labels? That way it would be cheap to just place labels on all bodies if I wanted to
      >- I can have point sizes, and later on I can have a large size and id-painting picking to allow cheap "raycasting" for bodies that are too small
      >- Would have to transform and upload points on every frame (and they'd need to be in view-space due to floating point precision), so it's hard to say what is more efficient. Maybe I can start with just another draw call of each meshcomponent, but I replace it with a 1 point mesh and use a flat color material?
      >- This is probably faster than using instanced drawing because I'd only upload the 2D coordinates of each point, instead of a full 4x4 matrix
      >- Draw points after meshes (but before skybox) and always fail the depth test
      >- Can store the last_position_drawn on mesh components after they're drawn, and on the pass afterward add those to the buffer 
      >- Actually the simplest would be to have N vertices at 0,0,0 in a buffer, and every frame upload a vec3 array of their positions as an uniform
          >- This doesn't work as each index in an array counts towards the "uniform vectors" limit for frag and vertex shaders, which are 1024 and 4095 respectively on Chrome for me. It could actually be the max uniform components limit instead, but honestly that's not so great either
> Sometimes it crashes due to an indexing error when filling in the color buffer?
>- Some entities had metadata components but not mesh components
>- Persistence
> - Save which scene was last opened and try loading it when opening again
<!-- # Remove pub use sub::* when I don't mean to, usually pub mod does what I want and keeps the namespace on the import path, which is neater -->
>- GLTF importer crate can't handle jpg images as it tries spawning a thread to decode it
    >- Had to change a feature to not use one of the inner dependencies of image-rs
>- We always try using vertex color, even if it contains garbage in it.. need to be able to set some additional defines like HAS_VERTEXCOLOR, HAS_TANGENTS, etc.
> How to do integration-based orbit prediction? Run the simulation N steps forward?
>- I really don't need "orbit prediction" unless I want to make maneuvre nodes like in KSP... what I really need is orbit trails, which is trivial to make: Store N last positions in a fixed time interval between them
>- There is no enforcing of a "root node", but inject_scene kind of expects it... I think I need some concept of a root node, or inject_scene needs to scan through all top-level entities and bake in the inject transform
    >- Scene injecting thing is gone
>- Okay-ish results on the simulation for now, we can improve later
> Good UV-mapped GLTF materials for the planets and asteroids (use blender?)
    >- I got some good CC4 textures from https://www.solarsystemscope.com/textures/, and will make custom shaders later
> Can probably fix the framerate thing by keeping a rolling sum of N frames and their times
>- Demo app has an x seconds per frame thing on the left which is just what I need
> Find a better way of handling component updates when sorting after reparenting
>- Reparenting thing is gone
>- The app will never know the full path of the GLB file when injecting, only if we keep track of the original URL ourselves (won't work for runtime uploads though...)
>- I have to revive the manifest thing and basically make sure all glb/texture assets have unique names, because if we hit a scene that was saved with assets that were uploaded at runtime all it will say is "albedo.png" and we won't know its folder
    >- This changed too much now, I don't think the issue is relevant anymore
> - What about that transform feedback thing? Maybe I can use that for line drawing or?
    >- It doesn't look like that helps line drawing. It's mainly for reusing vertex shader output for multiple draws
>- Can probably fix the initial "Request for pointer lock was denied because the document is not focused." by... just focusing the document on right click
    >- It seems fine on Chrome so I think I'll just ignore this for now
>- Expose a separate get_component and get_component_mut
>- Still get the rendering crash when building point color buffer
    >- Somehow the number of mesh and metadata components varies when I open a scene too, which shouldn't happen
        >- This is because I'm getting a random sample of asteroids/comets every time, so I get a random number of ones that don't have mass/radius and are skipped
    >- It was an OBOB on the previous fix
>- Can't set whether I want points or not in the settings dialog
<!-- - Basic solar system simulation at J2000 with bodies in the right size
    - Can't see the hover label when mousing over earth at 500x
    - Tracking phobos at planets_inner_moons and 500x shows it flickering... I think the camera transform update thing is not done at the right time
    - Watching it at higher speeds shows it move wrt camera when tracking, which should just not happen... I wonder if it's the physics thing?
    - It really seems like it's purely a camera reference thing. I think the reference is updated at start, after input is collected. We then update the body's position with physics.... on the next frame the camera will snap to the new location, etc. -->
>- Good sample scenes    
>- What's the point of saving state if I dump it every time any scene is loaded anyway?
<!-- - Compare that relative size
    - Seems good! -->
<!-- - I think I have to not use the localstorage or show a popup about storing data in the browser?
    - Disable it by default and add an option to save state in the settings
    - There's also this log level webpack thing? Where is that coming from?
        - It's just from webpack-dev-server, try packaging once to see if it shows up there too
        - Packaged: The loglevel thing doesn't show up, and all our data is persistent/cleared according to the setting like expected. Also, the entire app is like 4MB and all content, (including textures) is 100MB-ish now, which is not bad -->
>- Improve visuals a bit
    >- Correct-ish sun brightness
    >- Light test scene
        >- Put light intensity on body schema
        >- Add artificial candle body to test lights
        >- Plane is causing a material recompilation on every frame for default mat (still 60fps though!)
        >- There's no real way of allowing us to set an engine material via a body database (need to refactor it a bit)
        >- Get a gltf_metal_rough on that plane and see if we can see an intensity of 1 without tonemapping?
        >- Add a delta to prevent 0 distance to light source from leading to infinite illuminance
        >- What should be the units for light intensity that I send to the shaders? cd?
            >- https://google.github.io/filament/Filament.md.html#listing_fragmentexposure 
            >- Eq 57: E = (I / d2) * dot(n, l), and I is Luminous intensity, in cd        
        >- What is the unit of light intensity that is on the final framebuffer before tonemapping? (i.e. after lights were accumulated)
            >- Lout = f(v,l) * E, Lout being luminance in cd/m2, and f being the BSDF
        >- I'll neeed the section 4.7.2.2 to handle partially occluded sphere area lights: https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/course-notes-moving-frostbite-to-pbr-v2.pdf
            >- Maybe approximate the sphere area light as a point light if it's far enough that horizon problems aren't relevant?
        >- Pre-exposure seems to help with precision issues
            >- https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/course-notes-moving-frostbite-to-pbr-v2.pdf
        >- How to do exposure?
            >- https://google.github.io/filament/Filament.md.html#listing_fragmentexposure
            >- I can just set some EV100 value of like 700, and put it on a slider in the settings        
        >- Have to handle units for emissive colors, as I'll likely want to add some type of bloom later
        >- Maybe I just have a units change as things go to the shaders? Because or else I'll have a to use a factor of 1E12 every time I'm calculating square falloff for point lights
        >- Tonemapping:
            >- Will probably have to the full thing all at once or else store the lighting buffer in a float texture
``` C
// Computes the camera's EV100 from exposure settings
// aperture in f-stops
// shutterSpeed in seconds
// sensitivity in ISO
float exposureSettings(float aperture, float shutterSpeed, float sensitivity) {
    return log2((aperture * aperture) / shutterSpeed * 100.0 / sensitivity);
}

// Computes the exposure normalization factor from
// the camera's EV100
float exposure(ev100) {
    return 1.0 / (pow(2.0, ev100) * 1.2);
}

float ev100 = exposureSettings(aperture, shutterSpeed, sensitivity);
float exposure = exposure(ev100);

vec4 color = evaluateLighting();
color.rgb *= exposure;
```
>- Have exposure affect skybox
>- Fix Sun and Moon having default material        
>- Handle emissive material luminance correctly
    >- I'll just use the emissive factor for now, there doesn't seem to be a consensus on how to handle this
>- The GLTF test models are all gray now?
    >- Maybe it's exposure blowing up, I need to update the scene
    >- I was overriding their materials with blank gltf_metal_rough ones
>- Framebuffer refactor
>- Create a framebuffer that uses an f32 renderbuffer for the depth attachment (alongside with a regular color buffer)
>- Draw to it using logarithmic depth buffer by using the C function on the vertex shader and writing the depth on the fragment shader too
    >- WARNING: I don't currently need to write fragment depth from the pixel shader, but whenever I do I'll need to handle the logarithmic depth buffer values carefully: https://outerra.blogspot.com/2013/07/logarithmic-depth-buffer-optimizations.html
>- References:
    >- http://math.hws.edu/graphicsbook/c7/s4.html#:~:text=In%20WebGL%2C%20a%20framebuffer%20is,by%20the%20call%20to%20canvas.
    >- https://learnopengl.com/Advanced-OpenGL/Framebuffers
    >- https://webglfundamentals.org/webgl/lessons/webgl-render-to-texture.html
    >- https://webglfundamentals.org/webgl/lessons/webgl-framebuffers.html
    >- http://www.songho.ca/opengl/gl_fbo.html
>- I think the points are not always passing the depth test against the skybox
    >- This is actually a camera near/far precision problem
    >- I think I'll have to adjust near/far dynamically, but I always want them to be there though
>- Do something about near/far camera distance
    >- Maybe dynamically change this when close to a small body?
    >- With log depth buffer I can leave near at 0.0001 and it's fine
> - Fix bugs with disk generation
    > - Weird segment when we disable shared vertices for inner_radius != 0.0
    > - Inner_radius == 0.0 looks weird both for shared vertices and not
> - Setup double-sided rendering of meshes as a toggle
> - Fix disaster that is my scene schema
>     - Flatten nested hashmap by appending key names
>     - No. I'll very frequently want to "fetch all major bodies", and without separate maps this will be very awkward
> - Failing to parse wildcard number when there isn't one
> - There is more async behavior here than what I think there is... sometimes we finish parsing a database file only after we start loading the last scene, somehow
>     - I can work around this for now by just marking the resource as received after it was processed
> - Something is really messed up with scenes that load with wildcards, all meshes end up at NaN
>     - It's the physics system going nuts because all transforms end exactly at origin, so forces are infinite
>         - For some reason on the first pass the transforms are all identity, or even NaN for the barycenters
>         - It was getting into trouble because Venus/Mercury barycenters had 0 for mass and gravity calculations were spreading it around
>     - Camera also ends up at NaN somehow
> - Open Earth scene -> Reload page -> All parent entities are at NaN
>     - State vectors db finishes parsing after we try loading the scene
>     - Why do we get NaNs if we don't set a valid state vector? pos 0 should work...
> - Have to figure out what to do with Body List
> - Hoppocamp's mesh
>     - Bodies without mass or radius get assigned some default mesh
>- Figure out what to do with bounding boxes: If I click on a child mesh I want to defer back to selecting the parent entity, or no?
    > - Don't need to mess with this for now: I can defer selection in another way, and I can use radius fo goto
> - Actually allow children to have parents (doesn't really do anything but disable physics yet)
>     - Something is wrong with materials: It seems like only one instance of the material/textures can be used?
> - Angvel doesn't work so great: It seems to be spinning crazy fast every time
>     - It's the moment of inertia matrix thing
> - Rotation seems incorrect: Rotating Earth rotates the moons too
>     - RigidBodyComponent for N-body and free body physics
>     - KinematicComponent for on-track movement and set angular/linear velocities
>         - Later on we can use this to do on-track elliptical movement on orbits and stuff
> - Try out ring texture mapping
> - Allow double-sided mesh rendering
> - Rings
>     - Add rings to main bodies on default scenes
>     - It looks a bit wonky because I don't have the proper planet orientations yet but this will do

================================================================================

# TODO MVP
- Get rid of everything osculating elements *motion type*
    - I mostly want the n-body stuff and reconcyling both is a massive effort with no reward
    - Keep it in there in case I want to draw osculating orbits perhaps?
- Fix that bug where we can't save state with an entity selected, because entity ids are non deterministic
- Cleanup github repo and properly handle licensing like on my blog
    - MAYBE investigate async loading of assets before doing this because the skybox is way too slow

# TODO Bug fixes
- Need to flip the normals on the shaders if we're rendering double-sided
- Points don't have custom colors anymore?
- Can't see the text on 'Metal Rough Spheres No Textures' for some reason
- If a scene is loaded and it doesn't specify a focus target, it should be cleared instead
- Whenever we get a crash it just spams the console log with a million crashes because the global objects can't be borrowed again
- Figure out why my blender GLTF scenes somehow prevent all other GLTF files from loading
- GLTF test scene crashes when resetting scene
- Textures get reloaded when we reload/open new scenes
- Serializing entities kind of doesn't work because there's no guarantee the IDs will be the same. I think I need to serialize via body_id or name instead
- Customize the hover text on drag values whenever he adds it to egui
- Seems kind of weird to put Unit<> in scene description because I'm not sure what happens when deserializing it
- Real weird "headlight" effect when I get close to any mesh?
- Annoying spam of not finding the sampler for a gltf metal rough material even if we don't intend on specifying any texture
- Fix firefox dragging bug
- Position of labels when body is off screen is not correct, it flails everywhere
- Need better prediction of label size to prevent it from ever leaving the canvas
- Annoying bug where if you drag while moving the += movement_x() stuff will add to an invalid mouse_x as it never run, making it snap
    - Seems to only happen on firefox

================================================================================

# TODO Cleanup
- I think I'm doing some (or all) of the stuff in https://webgl2fundamentals.org/webgl/lessons/webgl-anti-patterns.html
- Remove bulk of code from mod.rs files and put them in e.g. interface/interface.rs instead, as it's easier to Ctrl+P to

# TODO UX
- I should be able to orbit the selection even if not focused...
- Let user upload his own scene ron files
    - Allow download of sample ron schema
- GUI to "add a body" with some state vectors/orbital parameters
- Use < and > keys to speed up and down
- Allow specifying move speed and other state settings for scenes (like paused/not, whether grid is on, etc.)
- Maybe pressing F/G without anything selected/focused would frame all of the objects in the scene

# TODO Visuals
- Relativistic effects of speed of travel: https://math.ucr.edu/home/baez/physics/Relativity/SR/Spaceship/spaceship.html
    - Not as hard as it seems.. mostly an FOV-like distortion and red-shifting
    - https://math.ucr.edu/home/baez/physics/Relativity/SR/penrose.html
    - https://jila.colorado.edu/~ajsh/insidebh/4dperspective.html
- Antialiasing: The default canvas framebuffer has antialiasing by default. Now that I setup my own framebuffer I'll need to enable it manually
- Custom material for earth that uses the light/day and cloud textures
- Planet trails for physics bodies
- Automatic exposure adjustment
- Actual IBL using the skybox
    - Massive cost for basically no gain
- Better line drawing
    - How did my JS app do it? The lines there seemed fine...
    - For main barycenter orbits the "object" still has center at the sun but the size is massive, so that using the camera as the origin doesn't help the precision issues
    - Line "tessellation" into triangles for thick lines
        - Geometry shaders? Not in WebGL2...
        - Will probably have to do a double-sided quad cross like old school foliage stuff
            - Would look like crap up close.. maybe if I added an extra uniform to vertex shaders to have them shrink when you approach
- Enable mipmap texture filtering: Not all formats support automatic generation of mips, so I need to check and enable certain extensions and fallback to linear if not available
- https://stackoverflow.com/questions/56829454/unable-to-generate-mipmap-for-half-float-texture
- Lerp when going to / point towards
- Hide points if they're too close to the camera (or else we can see them if we go inside the planet or if it's a GLTF model with offset origin)
- Shadows
- Use those constellation textures/grid to show them as overlays
- I can probably use 32bit depth buffer
- Logarithmic depth buffer
    - https://outerra.blogspot.com/2009/08/logarithmic-z-buffer.html
    - https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html
    - https://outerra.blogspot.com/2013/07/logarithmic-depth-buffer-optimizations.html
    - https://mynameismjp.wordpress.com/2010/03/22/attack-of-the-depth-buffer/
    - This would help when lowering the camera near distance and looking at far away orbits
    - Only required when drawing the ellipses
- Procedural textures and shaders for bodies (craters, weird shapes, textures, etc.)
- Bloom/post-processing
    - Would look great with the points stuff too
- Atmospheres
    - https://software.intel.com/content/www/us/en/develop/blogs/otdoor-light-scattering-sample-update.html

# TODO Optimizations
- Now that I have WebGL2 I can change to native handling of BGR instead of converting
- Apparently in WebGL2 there should be flags I could set to let opengl do the linear to srgb conversion on its own?
- Clipping of distant / out-of-frustum bodies

# TODO Physics / simulation
- I think my storing linear momentum I may be trashing my velocity precision, because all masses are like 10^24 and up
- Integration sample scenes with known 3-body patterns to test stability
- Setup reference scenes (3, 4, 5 masses, etc.) to compare results
- Cut down on calculations by ignoring some low mass - low mass interactions via some threshold
    - Mass doesn't change, I could do this easy by just sorting the entities by mass and stopping the inner loop after it hits N large masses, or something like that
- Rotation axes: 
    - https://astronomy.stackexchange.com/questions/18176/how-to-get-the-axial-tilt-vectorx-y-z-relative-to-ecliptic
    - Check the PDF I downloaded (Astronomy folder)
- Put orbital mechanics stuff in another crate once it's big enough    
- Make sure that skybox is aligned correctly with J2000 equatorial ecliptic reference system

# TODO Engine
- Salvage capability to display on-rails bodies and static orbits
    - Big scene with as many asteroids as I can handle
- Some way of specifying parameters for procedural meshes, or hashing the parameters, etc.
- Change picking to use a separate render pass by drawing IDs
    - Use a 1 pixel viewport to actually do the picking
    - Draw the dots mesh once again with a bigger point radius to allow picking bodies from far away
- Maybe use https://crates.io/crates/calloop for the event system instead
    - https://github.com/richardanaya/executor
- Maybe find a nicer way of having a generic component storage system
    - It should be easier now that I don't need the components to be serializable 
- Maybe investigate separate web worker thread with a shared memory buffer dedicated for the N-body stuff
- Better logging system that allows me to switch logging level for categories at a time

# TODO Content
- Earth satellites scene
    - Earth fixed in the very center
    - Moon
    - Different commercial satellites, starlink constellations, etc.

# TODO Devops
- Fix VSCode using html comment instead of quotes for markdown
- I think it should be possible to use chrome to debug WASM
    - Yes, but it doesn't have rust source maps yet, so good luck
- I think it should be possible to store the shaders in the public folder and tweak the automatic reloading stuff to allow hot-reloading of shaders
    - WasmPackPlugin has a "watchDirectories" member that I can use to exclude the public folder

================================================================================

# Other references:
- WebGL stats: https://webglreport.com/?v=2
- Local docs: file:///E:/Rust/system_viewer/target/wasm32-unknown-unknown/doc/system_viewer/index.html
- https://github.com/bevyengine/bevy
- https://github.com/not-fl3/macroquad
- https://github.com/hecrj/coffee
- https://github.com/ggez/ggez
- https://github.com/mrDIMAS/rg3d
- https://github.com/PistonDevelopers/piston
- https://github.com/wtvr-engine/wtvr3d/blob/8fcbb69104e0e204a6191723589013ddbd66629b/src/renderer/uniform.rs
- https://github.com/bridger-herman/wre/blob/6663afc6a5de05afe41d34f09422a7afcacb295c/src/frame_buffer.rs
- https://github.com/bridger-herman/wre/blob/6663afc6a5de05afe41d34f09422a7afcacb295c/resources/shaders/phong_forward.frag

# Physics references
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

# Orbital mechanics references
- Orbital mechanics equation cheat sheet even for non-elliptical orbits: http://www.bogan.ca/orbits/kepler/orbteqtn.html
- https://ssd.jpl.nasa.gov/horizons.cgi
- Great space SE threads:
    - https://space.stackexchange.com/questions/19322/converting-orbital-elements-to-cartesian-state-vectors
        - https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
    - https://space.stackexchange.com/questions/1904/how-to-programmatically-calculate-orbital-elements-using-position-velocity-vecto
    - https://space.stackexchange.com/questions/24276/why-does-the-eccentricity-of-venuss-and-other-orbits-as-reported-by-horizons
        - On this one they recommend using heliocentric if I intend to use osculating orbital elements, as they tend to be more consistent
    - https://space.stackexchange.com/questions/24276/why-does-the-eccentricity-of-venuss-and-other-orbits-as-reported-by-horizons
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