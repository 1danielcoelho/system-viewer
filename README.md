# Usage:
## Development  
Run this:
```
npm run dev
```
Then open http://localhost:9000/

## Deploying
Run this:
```
npm run build
```
The packaged build is fully contained within `/dist`. To deploy the build locally for a sanity check, do:
```
python -m http.server 8000
```
Then open http://localhost:8000/

# Features:
- WebGL2 (GLSL 300)
- Egui for UI
- wasm-pack and wasm-bindgen for Rust to Wasm 

# Conventions
- UV(0, 0) is on top left
- Right-handed, Z-up
- NDC space: Left-handed, X right, Y up, Z into the screen, (-1, -1, -1) on bottom left close, (1, 1, 1) on top right far


