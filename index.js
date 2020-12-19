const rust = import("./pkg/index");
const canvas = document.getElementById("rustCanvas");
const gl = canvas.getContext("webgl", { antialias: true });

function load_gltf(url, engine) {
  return fetch(url).then((response) =>
    response
      .arrayBuffer()
      .then((buffer) => engine.load_gltf(url, new Uint8Array(buffer)))
  );
}

function load_texture(url, engine) {
  return fetch(url).then((response) =>
    response
      .arrayBuffer()
      .then((buffer) => engine.load_texture(url, new Uint8Array(buffer)))
  );
}

rust.then(async (m) => {
  if (!gl) {
    alert("Failed to initialize WebGL");
    return;
  }

  let engine = new m.EngineInterface(document.getElementById("rustCanvas"));

  // Sync loading of all assets for now
  await Promise.all([
    load_gltf("./public/Duck.glb", engine),
    // load_gltf("./public/2CylinderEngine.glb", engine),
    // load_gltf("./public/gltf_3_cubes.glb", engine),
    load_texture("./public/shapes2_512.png", engine),
  ]);

  engine.begin_loop();
});
