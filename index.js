const rust = import("./pkg/index");
const garbage = import("./garbage");

rust.then(async (wasm_module) => {
    let gbg = await garbage;
    await gbg.run(wasm_module);
});
