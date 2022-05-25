#!/bin/sh
#If target is not web, it cannot load wasm.
cd wasm-paint
wasm-pack build -t web --release
cd ..