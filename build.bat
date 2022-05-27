@echo off

cargo build
wasm-bindgen --out-dir dist --target web --no-typescript target\wasm32-unknown-unknown\debug\system_viewer.wasm
echo f | xcopy /s /f /y www\index.html dist\index.html