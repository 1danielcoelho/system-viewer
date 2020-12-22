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
  await load_gltf("./public/Duck.glb", engine);
  //   await load_gltf("./public/2CylinderEngine.glb", engine);
  await load_gltf("./public/WaterBottle.glb", engine);
  //   await load_gltf("./public/DamagedHelmet.glb", engine);
  //   await load_gltf("./public/gltf_3_cubes.glb", engine);
  await load_texture("./public/shapes2_512.png", engine);
  //   await load_texture("./public/WaterBottle_baseColor.png", engine);
  //   await load_texture("./public/WaterBottle_emissive.png", engine);
  //   await load_texture("./public/WaterBottle_normal.png", engine);
  //   await load_texture(
  //     "./public/WaterBottle_occlusionRoughnessMetallic.png",
  //     engine
  //   );

  engine.begin_loop();
});
