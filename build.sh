#!/bin/sh
#If target is not web, it cannot load wasm.
wasm-pack build -t web
