# Break off the mesh stuff from material
- Class for mesh
- Class for object
    - Derive from it to make a MeshObject
    - Derive from it to make an UIObject
    - Continue reading this: https://doc.rust-lang.org/stable/book/ch17-02-trait-objects.html
    - and this: https://old.reddit.com/r/rust/comments/9h1xy8/how_to_avoidtranslate_oopalike_thinking_when/
    - Vec<Box<dyn BaseTrait>>. 

# Setup keyboard and mouse controls to have a basic look around
- Maybe have some UI widget to control camera parameters like FOV, near/far planes, etc.
# Setup basic objects for grid and coordinate axes
# Get some better models and obj/gltf parsing
- Show coordinates on screen
# I think I'll need wasm-bindgen-futures
<!-- # Setup a time variable and animate a material rotation -->
<!-- # Why is the cube rendering at the bottom left?
- Missing viewport -->