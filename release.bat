@echo off
cd wasm-paint
wasm-pack build -t web --release -- --features font
cd ..
