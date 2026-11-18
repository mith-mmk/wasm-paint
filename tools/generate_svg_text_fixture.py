#!/usr/bin/env python3
"""Generate the synthetic SVG text integration font; requires fontTools."""
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import newTable
from fontTools.ttLib.tables.S_V_G_ import SVGDocument


def document(glyph, body):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128">'
        f'<g id="glyph{glyph}">{body}</g></svg>'
    )


out = Path(__file__).resolve().parents[1] / "paintcore/tests/fixtures/svg-text.ttf"
names = [".notdef", "letterA", "letterB"]
builder = FontBuilder(128, isTTF=True)
builder.font.recalcTimestamp = False
builder.font["head"].created = builder.font["head"].modified = 2082844800
builder.setupGlyphOrder(names)
builder.setupCharacterMap({65: "letterA", 66: "letterB"})
builder.setupGlyf({name: TTGlyphPen(None).glyph() for name in names})
builder.setupHorizontalMetrics({name: (64, 0) for name in names})
builder.setupHorizontalHeader(ascent=128, descent=0)
builder.setupNameTable({
    "familyName": "SVG Text Test",
    "styleName": "Regular",
    "uniqueFontIdentifier": "SvgTextTest",
    "fullName": "SVG Text Test",
    "psName": "SvgTextTest",
})
builder.setupOS2(sTypoAscender=128, sTypoDescender=0, usWinAscent=128, usWinDescent=0)
builder.setupPost()
table = newTable("SVG ")
table.docList = [
    SVGDocument(document(1, '<g opacity=".5"><rect x="8" y="16" width="32" height="32" fill="#2277cc"/></g>'), 1, 1),
    SVGDocument(document(2, '<defs><filter id="f"><feGaussianBlur stdDeviation="1"/></filter></defs><rect filter="url(#f)" width="32" height="32"/>'), 2, 2),
]
builder.font["SVG "] = table
builder.save(out)
