#!/usr/bin/env python3
"""Embed the separately licensed fontcore fixtures into synthetic fonts.
Requires fontTools/CairoSVG/Pillow. No download. Arguments: FONTCORE_DIR OUTPUT_DIR.
The Noto case is synthetic, not an official SVG font. Twemoji artwork was extracted
from a real font; this minimal wrapper exercises the public load/shape/draw route.
"""
import sys, json
from pathlib import Path
import xml.etree.ElementTree as ET
import cairosvg
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import newTable
from fontTools.ttLib.tables.S_V_G_ import SVGDocument
src,out = map(Path,sys.argv[1:])
out.mkdir(parents=True,exist_ok=True)
for name,filename,em,y in [('noto','noto-emoji_u1f600.svg',128,0),('twemoji','twemoji-glyph5.svg',2048,1700)]:
    svg=(src/'tests/fixtures/svg-extraction'/filename).read_text()
    fb=FontBuilder(em,isTTF=True)
    names=['.notdef','unused1','unused2','unused3','unused4','glyph5']
    fb.setupGlyphOrder(names)
    fb.setupCharacterMap({65:'glyph5'})
    fb.setupGlyf({n:TTGlyphPen(None).glyph() for n in names})
    fb.setupHorizontalMetrics({n:(em,0) for n in names})
    fb.setupHorizontalHeader(ascent=em,descent=0)
    fb.setupNameTable({'familyName':'SVG extraction probe','styleName':'Regular','uniqueFontIdentifier':'probe','fullName':'SVG extraction probe','psName':'SVGExtractionProbe'})
    fb.setupOS2(sTypoAscender=em,sTypoDescender=0,usWinAscent=em,usWinDescent=0)
    fb.setupPost(); fb.setupMaxp()
    table=newTable('SVG '); table.docList=[SVGDocument(svg,5,5)]; fb.font['SVG ']=table
    fb.save(out/f'{name}.ttf')
    root=ET.fromstring(svg)
    root.set('width',str(em));root.set('height',str(em));root.set('overflow','visible')
    document=f'<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 {-y} {em} {em}">{ET.tostring(root,encoding="unicode")}</svg>'
    cairosvg.svg2png(bytestring=document.encode(),write_to=str(out/f'{name}-reference.png'))
print(json.dumps({'fontcore':str(src),'output':str(out),'twemoji_y_origin':1700/2048*256}))
