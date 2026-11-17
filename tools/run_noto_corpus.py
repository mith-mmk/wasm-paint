#!/usr/bin/env python3
"""Pinned Noto SVG assets -> synthetic SFNT -> typed font context -> paintcore.
Usage: run_noto_corpus.py FONTCORE_DIR OUTPUT_DIR [--download] [--check]
Requires fontTools, CairoSVG, Pillow, cargo. Downloads only on explicit --download.
Generated fonts, references and renderer outputs retain Noto's Apache-2.0 license.
This samples 20 artworks, not the entire Noto corpus or official Noto SVG fonts.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import urllib.request
import xml.etree.ElementTree as ET
import cairosvg
from PIL import Image
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import newTable
from fontTools.ttLib.tables.S_V_G_ import SVGDocument
fontcore, out = (Path(p).resolve() for p in sys.argv[1:3])
repo = Path(__file__).resolve().parent.parent
manifest = json.loads((repo / 'tools/noto-corpus.json').read_text())
assets = out / 'corpus'
assets.mkdir(parents=True,exist_ok=True)
runner = out / 'context-runner'
runner.mkdir(parents=True,exist_ok=True)
q=lambda p:json.dumps(str(p))
(runner / 'Cargo.toml').write_text(f'''[package]
name = "paintcore-svg-context-probe"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
paintcore = {{ path = {q(repo/'paintcore')}, features = ["svg-font"] }}
fontcore = {{ path = {q(fontcore)}, features = ["svg-fonts"] }}
[patch.crates-io]
fontcore = {{ path = {q(fontcore)} }}
[[bin]]
name = "svg-context-probe"
path = {q(repo/'tools/svg_context_probe.rs')}
''')
results=[]
for item in manifest['files']:
    source=assets/item['name']
    if '--download' in sys.argv[3:] and not source.exists():
        source.write_bytes(urllib.request.urlopen(manifest['source_base']+item['name']).read())
    data=source.read_bytes()
    if hashlib.sha256(data).hexdigest()!=item['sha256']:
        raise ValueError(f'checksum mismatch: {source}')
    prefix=assets/source.stem
    fb=FontBuilder(128,isTTF=True)
    names=['.notdef','unused1','unused2','unused3','unused4','glyph5']
    fb.setupGlyphOrder(names);fb.setupCharacterMap({65:'glyph5'})
    fb.setupGlyf({n:TTGlyphPen(None).glyph() for n in names})
    fb.setupHorizontalMetrics({n:(128,0) for n in names})
    fb.setupHorizontalHeader(ascent=128,descent=0)
    fb.setupNameTable({'familyName':'Noto corpus probe','styleName':'Regular','uniqueFontIdentifier':'probe','fullName':'Noto corpus probe','psName':'NotoCorpusProbe'})
    fb.setupOS2(sTypoAscender=128,sTypoDescender=0,usWinAscent=128,usWinDescent=0)
    fb.setupPost();fb.setupMaxp()
    table=newTable('SVG ');table.docList=[SVGDocument(data.decode(),5,5)];fb.font['SVG ']=table
    fb.save(str(prefix)+'.ttf')
    reference=str(prefix)+'-reference.png'
    # Canonicalize verified sources only for the reference. This expands the
    # fixed Adobe internal namespace entities without enabling Cairo unsafe/IO.
    # The synthetic font retains the original bytes, including any DTD.
    canonical = ET.tostring(ET.fromstring(data))
    cairosvg.svg2png(bytestring=canonical,output_width=256,output_height=256,write_to=reference)
    command = [os.environ['SVG_CONTEXT_PROBE_BINARY']] if 'SVG_CONTEXT_PROBE_BINARY' in os.environ else ['cargo','run','--quiet','--manifest-path',str(runner/'Cargo.toml'),'--']
    run=subprocess.run(command+[str(prefix)+'.ttf',str(prefix),'0'],capture_output=True,text=True)
    result={'file':source.name,'rendered':run.returncode==0}
    try:result['probe']=json.loads(run.stdout)
    except ValueError:result['stdout']=run.stdout
    if run.returncode:
        result['error']=run.stderr.strip()
    else:
        a=Image.open(str(prefix)+'-all.png').convert('RGBA').tobytes()
        b=Image.open(reference).convert('RGBA').tobytes()
        result['mae']=sum(abs(x-y) for x,y in zip(a,b))/len(a)
        result['coverage_delta']=sum(x>0 for x in a[3::4])-sum(x>0 for x in b[3::4])
        result['reference_nontransparent_pixels']=sum(x>0 for x in b[3::4])
        result['comparison_pass']=result['mae']<=1.0 and abs(result['coverage_delta'])<=256
    results.append(result)
    print(source.name, 'OK' if result.get('comparison_pass') else 'FAIL',result.get('mae',result.get('error')),flush=True)
summary={'source_revision':manifest['source_revision'],'fontcore_commit':subprocess.check_output(['git','-C',str(fontcore),'rev-parse','HEAD'],text=True).strip(),'results':results}
(out/'noto-corpus-results.json').write_text(json.dumps(summary,indent=2)+'\n')

if '--check' in sys.argv[3:]:
    expected = {f['name']: f['expected'] for f in manifest['files']}
    for result in results:
        if expected[result['file']] == 'render':
            assert result.get('comparison_pass'), f"{result['file']}: rendering/reference regression"
        else:
            assert not result['rendered'], f"{result['file']}: unsupported input must not succeed"
    print('corpus gate passed: supported references and explicit unsupported failures')
