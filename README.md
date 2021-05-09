# What is this?
[![Demo image](/demo.gif)](/demo.gif)

This is a hobby, tiny 3D engine targetting WASM and the browser, capable of doing N-body physics simulations.

The main goal is to be able to quickly visualize and move around solar systems in the correct scale, and simulate their motion and lighting in a natural way. Also this was a way for me to practice Rust, Entity-Component systems and targetting the WebAssembly ecosystem.

I tried to stick to trusted source data and approximations for body physical parameters, so most of this data comes from NASA's HORIZONS system and JPL's Small-Body Database Search Engine, and when required evolved to the J2000 epoch using the mean orbital elements. It uses a bunch of Python scripts (see `/scripts`) to download/process all of this data and combine it into a set of large .json files (`/public/database`), which are read by the app. The source data downloaded from NASA is not on this repo, but the final database files are.

I have many cool plans for this like visualizing asteroids/comets in the solar system, artificial satellites around the Earth and doing physically-correct relativistic motion effects. Progress doesn't go as fast as I would like though, as this is just one of many side projects.

# Sources and references
- [CC4](https://creativecommons.org/licenses/by/4.0/) solar system body textures from here: https://www.solarsystemscope.com/textures/
- Starmap from the Hipparcos-2, Tycho-2 and Gaia Data Release 2: https://svs.gsfc.nasa.gov/4851
- https://360toolkit.co/ to convert the cubemap images
- [`egui_web`](https://crates.io/crates/egui_web) as a source of the `gui_backend` inner crate

# Usage:
## Running 
Check it out on your browser by clicking [here](https://1danielcoelho.github.io/assets/system-viewer/index.html)!

It's still a work-in-progress though, and it may stutter a little big while it downloads all the high-resolution textures.

## Development  
Run this:
```
git clone https://github.com/1danielcoelho/system-viewer
cd system-viewer
npm install
npm run dev
```
It should open the browser right away. If not you can open http://localhost:9000/ manually.

Within the app, make sure you go to Settings and `Allow Local Storage` if you wish: That will allow it to store app state on your browser, as well as the last opened scene.

See the `/schemas` folder for schemas describing the format for the database and scene description files. New scenes can be added to the `/public/scenes` folder and hot-reloaded, but will need to also be listed on the `/public/scenes/auto_load_manifest.txt` file. 

## Deployment
Run this:
```
git clone https://github.com/1danielcoelho/system-viewer
cd system-viewer
npm install
npm run build
```
The packaged build is fully contained within `/dist`. To deploy the build locally for a sanity check, run this from `/dist`:
```
python -m http.server 8000
```
Then open http://localhost:8000/

## Source data
This part is not meant to be done by users/dev, but if you have the source data on your drive, run `/scripts/main.py`. There are a few hard-coded paths within the files pointing to where that data is, and those would need to be manually edited. 
Some of the source data can be downloaded with the `download_horizons_files.py` script.

# Features:
- WebGL2 (GLSL 300);
- [Egui](https://github.com/emilk/egui) for immediate-mode UI;
- wasm-pack and wasm-bindgen for Rust to WASM;
- Full, custom entity-component-system for Data Oriented Design;
- Semi-implicit Euler integration for N-body (may be switched to RK4 eventually).

# Conventions
- UV(0, 0) is on top left
- Right-handed, Z-up
- NDC space: Left-handed, X right, Y up, Z into the screen, (-1, -1, -1) on bottom left close, (1, 1, 1) on top right far


