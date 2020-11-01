const rust = import('./pkg/index');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true });

rust.then(m => {
    if (!gl) {
        alert('Failed to initialize WebGL');
        return;
    }

    // let req = new XMLHttpRequest();
    // req.open("GET", "./public/Duck.glb", true);
    // req.responseType = "arraybuffer";
    // req.onload = function (ev) {
    //     m.load_gltf(new Uint8Array(req.response));
    // }
    // req.send();
    
    m.initialize();
});
