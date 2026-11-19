#!/usr/bin/env python3
"""Synthetic compositing/use correctness cases, explicitly not Noto artwork.
Usage: run_svg_semantics.py OUTPUT_DIR [--check]
Requires CairoSVG/Pillow and cargo. Pinned resvg is the independent primary reference;
CairoSVG is also measured, including its documented opacity discrepancies.
"""
import json
from pathlib import Path
import subprocess
import sys
import cairosvg
from PIL import Image
repo=Path(__file__).resolve().parent.parent
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
runner=out/'resvg-reference'
runner.mkdir(exist_ok=True)
(runner/'Cargo.toml').write_text(f'''[package]
name = "paintcore-independent-svg-reference"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
resvg = {{ version = "=0.45.1", default-features = false }}
[[bin]]
name = "resvg-reference"
path = {json.dumps(str(repo/'tools/resvg_reference.rs'))}
''')
results=[]
for svg in sorted((repo/'paintcore/tests/fixtures/svg-render').glob('*.svg')):
    actual=out/(svg.stem+'-all.png');reference=out/(svg.stem+'-reference.png');cairo=out/(svg.stem+'-cairo.png')
    cairosvg.svg2png(bytestring=svg.read_bytes(),output_width=256,output_height=256,write_to=str(cairo))
    subprocess.run(['cargo','run','--quiet','--manifest-path',str(runner/'Cargo.toml'),'--',str(svg),str(reference)],check=True)
    r=subprocess.run(['cargo','run','--quiet','-p','paintcore','--features','svg-font','--example','svg_document_probe','--',str(svg),str(actual),'2','0'],cwd=repo,capture_output=True,text=True)
    item={'fixture':svg.name,'rendered':r.returncode==0}
    if r.returncode: item['error']=r.stderr.strip()
    else:
        a=Image.open(actual).convert('RGBA').tobytes();b=Image.open(reference).convert('RGBA').tobytes()
        c=Image.open(cairo).convert('RGBA').tobytes()
        item['cairo_mae']=sum(abs(x-y) for x,y in zip(a,c))/len(a)
        item['mae']=sum(abs(x-y) for x,y in zip(a,b))/len(a)
        item['coverage_delta']=sum(x>0 for x in a[3::4])-sum(x>0 for x in b[3::4])
        item['comparison_pass']=item['mae']<=1 and abs(item['coverage_delta'])<=256
    print(item,flush=True);results.append(item)
(out/'svg-semantics-results.json').write_text(json.dumps({'primary_reference':'resvg 0.45.1','secondary_CairoSVG':cairosvg.__version__,'synthetic':True,'results':results},indent=2)+'\n')
if '--check' in sys.argv[2:]:
    assert all(r.get('comparison_pass') for r in results), 'SVG semantic reference regression'
