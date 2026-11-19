#!/usr/bin/env python3
"""Report exact RGBA differences, not a renderer-conformance pass threshold."""
import json
import sys
from pathlib import Path
from PIL import Image

out = Path(sys.argv[1])
results = {}
for name in ('noto', 'twemoji'):
    actual = Image.open(out / f'{name}-all.png').convert('RGBA')
    reference = Image.open(out / f'{name}-reference.png').convert('RGBA')
    assert actual.size == reference.size
    a, b = actual.tobytes(), reference.tobytes()
    results[name] = {
        'probe': json.loads((out / f'{name}.json').read_text()),
        'reference_nontransparent_pixels': sum(b[i] != 0 for i in range(3, len(b), 4)),
        'mean_absolute_rgba_error_0_255': sum(abs(x-y) for x,y in zip(a,b)) / len(a),
        'different_rgba_pixels': sum(a[i:i+4] != b[i:i+4] for i in range(0, len(a), 4)),
    }
print(json.dumps(results, indent=2))

if '--check' in sys.argv[2:]:
    for name, result in results.items():
        rendered = result['probe']['renders'][0]
        assert rendered['ok'], f'{name}: rendering failed'
        assert result['reference_nontransparent_pixels'] > 0
        assert abs(rendered['nontransparent_pixels'] - result['reference_nontransparent_pixels']) <= 256, f'{name}: coverage regression'
        assert result['mean_absolute_rgba_error_0_255'] <= 1.0, f'{name}: RGBA regression'
