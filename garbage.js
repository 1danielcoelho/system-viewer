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

function load_text(url, engine) {
  return fetch(url).then((response) =>
    response.text().then((text) => engine.load_ephemerides(url, text))
  );
}

export function test_garbage() {
  const canvas = document.getElementById("rustCanvas");
  let wasm_module = canvas.hack;

  console.log("test garbage:", wasm_module);

  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = ".json";

  fileInput.addEventListener("change", (e) => {
    if (fileInput.files && fileInput.files[0]) {
      const file = fileInput.files[0];

      const reader = new FileReader();
      reader.onload = async (loadEvent) => {
        const readJson = loadEvent.target.result;
        console.log("read", readJson);
        wasm_module.load_ephemerides_external(
          "./public/ephemerides/3@sun.txt",
          readJson
        );
      };
      reader.readAsText(file);
    }
  });

  fileInput.click();
}

export async function run(wasm_module) {
  const canvas = document.getElementById("rustCanvas");
  const gl = canvas.getContext("webgl2", { antialias: true });

  if (!gl) {
    alert("Failed to initialize WebGL");
    return;
  }

  console.log("before run", canvas.hack);
  wasm_module.run();
  canvas.hack = wasm_module;
  console.log("after run", canvas.hack);

  //   console.log("engine before", engine);
  //   engine = new wasm_module.EngineInterface(canvas);
  //   canvas.engine = engine;
  //   console.log("engine is now", canvas.engine);

  //   console.log("calling test garbage");
  //   test_garbage();

  // Sync loading of all assets for now
  //   await load_text("./public/ephemerides/3@sun.txt", engine);

  //   await load_gltf("./public/Duck.glb", engine);
  //   await load_gltf("./public/2CylinderEngine.glb", engine);
  //   await load_gltf("./public/WaterBottle.glb", engine);
  //   await load_gltf("./public/DamagedHelmet.glb", engine);
  //   await load_gltf("./public/BoomBox.glb", engine);
  //   await load_gltf("./public/Box.glb", engine);
  //   await load_gltf("./public/gltf_3_cubes.glb", engine);
  //   await load_texture("./public/shapes2_512.png", engine);
  //   await load_texture("./public/WaterBottle_baseColor.png", engine);
  //   await load_texture("./public/WaterBottle_emissive.png", engine);
  //   await load_texture("./public/WaterBottle_normal.png", engine);
  //   await load_texture(
  //     "./public/WaterBottle_occlusionRoughnessMetallic.png",
  //     engine
  //   );

  //   await engine.begin_loop();
}
