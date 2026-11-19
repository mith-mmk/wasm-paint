#!/usr/bin/env python3
"""Exercise an explicitly supplied unreleased fontcore checkout without pinning it.
Usage: run_svg_context_probe.py FONTCORE_DIR OUTPUT_DIR
Run prepare_svg_probe.py first. Requires cargo on PATH. Build outputs obey
CARGO_TARGET_DIR; generated runner/Cargo.lock live in ignored OUTPUT_DIR.
"""
from pathlib import Path
import json
import subprocess
import sys
fontcore, out = (Path(p).resolve() for p in sys.argv[1:])
repo = Path(__file__).resolve().parent.parent
runner = out / 'context-runner'
runner.mkdir(parents=True, exist_ok=True)
q = lambda p: json.dumps(str(p))
(runner / 'Cargo.toml').write_text(f'''[package]
name = "paintcore-svg-context-probe"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
paintcore = {{ path = {q(repo / 'paintcore')}, features = ["svg-font"] }}
fontcore = {{ path = {q(fontcore)}, features = ["svg-fonts"] }}
[patch.crates-io]
fontcore = {{ path = {q(fontcore)} }}
[[bin]]
name = "svg-context-probe"
path = {q(repo / 'tools/svg_context_probe.rs')}
''')
for name, y in [('noto', '0'), ('twemoji', '212.5')]:
    result = subprocess.run(['cargo', 'run', '--quiet', '--manifest-path', str(runner / 'Cargo.toml'), '--', str(out / f'{name}.ttf'), str(out / name), y], check=True, capture_output=True, text=True)
    (out / f'{name}.json').write_text(result.stdout)
    print(result.stderr, end='', file=sys.stderr)
    print(name, result.stdout.strip())
