const rust = import('./pkg/index');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true });

rust.then(m => {
    if (!gl) {
        alert('Failed to initialize WebGL');
        return;
    }

    let canvas = document.getElementById('rustCanvas');

    let engine = new m.EngineInterface(canvas);

    // TODO: This will crash, as it will complete after engine.begin_loop() is called, which consumes the engine
    // let req = new XMLHttpRequest();
    // req.open("GET", "./public/Duck.glb", true);
    // req.responseType = "arraybuffer";
    // req.onload = function (ev) {
    //     engine.load_gltf(new Uint8Array(req.response));
    // }
    // req.send();

    engine.begin_loop();
});
