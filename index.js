import("./pkg/index").then(async (wasm_module) => {
  const canvas = document.getElementById("rustCanvas");
  const gl = canvas.getContext("webgl2", { antialias: true });

  if (!gl) {
    alert("Failed to initialize WebGL");
    return;
  }

  // HACK: The greatest hack known to man involves stuffing our loaded wasm module into the canvas
  // so that from wasm we can call javascript functions with callbacks. When they complete, they
  // can independently fetch the loaded wasm module from the same canvas, and provide new data to the engine.

  // Until winit supports async event loops, this seems to be the only way of "returning control to JS and later do something":
  // The standard way is to have the main wasm entrypoint async and just await whenever we want to return to JS, which is what
  // everyone does and why it's not a big issue. Given how the winit event loop can't be in an async function yet however,
  // (https://github.com/rust-windowing/winit/issues/1199), this is all we can do.
  // We can't even have the file callbacks inside wasm even though all the types are available and work fine, because to load in the
  // new data we have to mut borrow the engine, and if the callstack originates from the winit event loop where we're
  // doing a regular engine.update and drawing the egui UI, the engine will already be borrowed, and it will panic.

  // Note how we no longer export the engine interface struct: The engine is a thread local static to the
  // wasm module now, because in order to do this "putting the engine in the canvas" trick it can't be fully
  // moved into the winit event loop closure. If it's moved in there, the next time we try accessing it from outside
  // the wasm module (e.g. from a JS callback) it would just panic saying we tried to access a moved value.
  wasm_module.initialize();
  wasm_module.start_loop();  // Note how we don't await this async function here: This is impotant, but I don't know why...
  window.wasm_module = wasm_module;
});
