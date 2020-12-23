# Building:
- To support clipboard stuff (unstable) you need to add "--cfg=web_sys_unstable_apis" to the RUSTFLAGS environment variable
    - set RUSTFLAGS=--cfg=web_sys_unstable_apis
    - So far I haven't found a way of doing this automatically

# Features:
- WebGL2 (GLSL 300)
- Egui 0.5.0 for UI
- wasm-pack and wasm-bindgen for Rust to Wasm 

# Conventions
- UV(0, 0) is on top left
- Right-handed, Z-up


