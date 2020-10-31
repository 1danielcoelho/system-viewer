const rust = import('./pkg/index');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true });

rust.then(m => {
    if (!gl) {
        alert('Failed to initialize WebGL');
        return;
    }

    let req = new XMLHttpRequest();
    req.open("GET", "./public/Duck.gltf", true);
    req.responseType = "text";
    req.onload = function (ev) {
        console.log('loaded', req.response);
    }
    req.send();
    
    m.initialize();
});
