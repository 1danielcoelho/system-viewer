const rust = import('./pkg/system_viewer');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true });

rust.then(m => {
    if (!gl) {
        alert('Failed to initialize WebGL');
        return;
    }
    
    const viewer = new m.Viewer();
    viewer.start();    
});
