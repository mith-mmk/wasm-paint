import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const TEST_HTML_URL = new URL('../test.html', import.meta.url);

let universe;
let canvasWidth = 0;

function parseAttributes(text) {
  const attributes = {};
  const matches = text.matchAll(/([a-zA-Z_:][-a-zA-Z0-9_:]*)\s*=\s*"([^"]*)"/g);
  for (const match of matches) {
    attributes[match[1].toLowerCase()] = match[2];
  }
  return attributes;
}

function formatNumber(value) {
  return Number.parseFloat(value.toFixed(3)).toString();
}

function parsePathData(pathData, viewBox, renderWidth, renderHeight) {
  const tokens = pathData.match(/[MLCQZmlcqz]|[-+]?(?:\d*\.\d+|\d+)(?:e[-+]?\d+)?/g) ?? [];
  const [viewX, viewY, viewWidth, viewHeight] = viewBox;
  const sx = renderWidth / viewWidth;
  const sy = renderHeight / viewHeight;
  const output = [];

  let index = 0;
  let currentX = 0;
  let currentY = 0;
  let startX = 0;
  let startY = 0;

  const nextNumber = (command) => {
    if (index >= tokens.length) {
      throw new Error(`missing numeric argument for ${command}`);
    }
    const value = Number.parseFloat(tokens[index]);
    index += 1;
    if (Number.isNaN(value)) {
      throw new Error(`invalid numeric argument for ${command}`);
    }
    return value;
  };

  const pushPoint = (x, y) => {
    output.push(formatNumber((x - viewX) * sx));
    output.push(formatNumber((y - viewY) * sy));
  };

  while (index < tokens.length) {
    const command = tokens[index];
    index += 1;

    switch (command) {
      case 'M':
      case 'm': {
        let x = nextNumber(command);
        let y = nextNumber(command);
        if (command === 'm') {
          x += currentX;
          y += currentY;
        }
        currentX = x;
        currentY = y;
        startX = x;
        startY = y;
        output.push('M');
        pushPoint(x, y);
        break;
      }
      case 'L':
      case 'l': {
        let x = nextNumber(command);
        let y = nextNumber(command);
        if (command === 'l') {
          x += currentX;
          y += currentY;
        }
        currentX = x;
        currentY = y;
        output.push('L');
        pushPoint(x, y);
        break;
      }
      case 'C':
      case 'c': {
        let cx1 = nextNumber(command);
        let cy1 = nextNumber(command);
        let cx2 = nextNumber(command);
        let cy2 = nextNumber(command);
        let ex = nextNumber(command);
        let ey = nextNumber(command);
        if (command === 'c') {
          cx1 += currentX;
          cy1 += currentY;
          cx2 += currentX;
          cy2 += currentY;
          ex += currentX;
          ey += currentY;
        }
        currentX = ex;
        currentY = ey;
        output.push('C');
        pushPoint(cx1, cy1);
        pushPoint(cx2, cy2);
        pushPoint(ex, ey);
        break;
      }
      case 'Q':
      case 'q': {
        let cx = nextNumber(command);
        let cy = nextNumber(command);
        let ex = nextNumber(command);
        let ey = nextNumber(command);
        if (command === 'q') {
          cx += currentX;
          cy += currentY;
          ex += currentX;
          ey += currentY;
        }
        currentX = ex;
        currentY = ey;
        output.push('Q');
        pushPoint(cx, cy);
        pushPoint(ex, ey);
        break;
      }
      case 'Z':
      case 'z':
        currentX = startX;
        currentY = startY;
        output.push('Z');
        break;
      default:
        throw new Error(`unsupported SVG path command ${command}`);
    }
  }

  return output.join(' ');
}

function parseTestHtml(html, width) {
  const scale = 2.0;
  const gap = 24;
  const margin = 24;
  const glyphs = [];
  const svgMatches = html.matchAll(/<svg\b([^>]*)>([\s\S]*?)<\/svg>/gi);

  let x = margin;
  let y = margin;
  let rowHeight = 0;
  let rows = 0;
  let rowHasGlyph = false;

  for (const match of svgMatches) {
    const svgAttributes = parseAttributes(match[1]);
    const widthPx = Number.parseFloat(svgAttributes.width ?? '0') * scale;
    const heightPx = Number.parseFloat(svgAttributes.height ?? '0') * scale;
    if (!Number.isFinite(widthPx) || !Number.isFinite(heightPx) || widthPx <= 0 || heightPx <= 0) {
      continue;
    }

    const viewBoxText = svgAttributes.viewbox ?? svgAttributes.viewBox ?? `0 0 ${widthPx} ${heightPx}`;
    const viewBox = viewBoxText
      .trim()
      .split(/\s+/)
      .map((value) => Number.parseFloat(value));
    if (viewBox.length !== 4 || viewBox.some((value) => Number.isNaN(value))) {
      continue;
    }

    const pathMatches = match[2].matchAll(/<path\b([^>]*)\/?>/gi);
    const layers = [];
    for (const pathMatch of pathMatches) {
      const pathAttributes = parseAttributes(pathMatch[1]);
      if (!pathAttributes.d) {
        continue;
      }

      const layer = {
        commands: parsePathData(pathAttributes.d, viewBox, widthPx, heightPx),
        fillRule: 'nonzero',
      };

      if (pathAttributes.fill && pathAttributes.fill !== 'currentColor' && pathAttributes.fill !== 'none') {
        const normalized = pathAttributes.fill.trim();
        if (/^#([0-9a-f]{6})$/i.test(normalized)) {
          layer.color = Number.parseInt(normalized.slice(1), 16);
        }
      }

      layers.push(layer);
    }

    if (layers.length === 0) {
      continue;
    }

    if (x + widthPx + margin > width && rowHasGlyph) {
      x = margin;
      y += rowHeight + gap;
      rowHeight = 0;
      rowHasGlyph = false;
    }

    if (!rowHasGlyph) {
      rows += 1;
      rowHasGlyph = true;
    }

    glyphs.push({ x, y, layers });
    x += widthPx + gap;
    rowHeight = Math.max(rowHeight, heightPx);
  }

  const usedHeight = y + rowHeight + margin;
  return {
    glyphs,
    rows: rows === 0 && glyphs.length > 0 ? 1 : rows,
    scale,
    usedHeight,
  };
}

async function workerInit(width, height) {
  await init();
  universe = new Universe(width, height);
  universe.clear(0xffffff);
  const image = universe.getImageData(0);
  postMessage({ message: 'init', image });
}

async function renderGlyphSheet() {
  if (universe == null) {
    return;
  }

  const html = await fetch(TEST_HTML_URL).then((response) => response.text());
  const parsed = parseTestHtml(html, canvasWidth);

  universe.clear(0xffffff);
  universe.drawGlyphs(parsed.glyphs, 0x111111);
  universe.combine();

  const image = universe.getImageData(0);
  postMessage({
    message: 'run',
    image,
    summary: {
      source: 'tests/test.html',
      count: parsed.glyphs.length,
      rows: parsed.rows,
      scale: parsed.scale,
      usedHeight: parsed.usedHeight,
    },
  });
}

onmessage = async function(ev) {
  const data = ev.data;
  if (data.command == null) {
    return;
  }

  try {
    switch (data.command) {
      case 'init':
        canvasWidth = data.width;
        await workerInit(canvasWidth, data.height);
        break;
      case 'run':
        await renderGlyphSheet();
        break;
      default:
        break;
    }
  } catch (error) {
    postMessage({
      message: 'error',
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
