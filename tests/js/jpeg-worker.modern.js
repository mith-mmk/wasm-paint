self.postMessage({ type: 'test' });

import init, { Universe } from '../../wasm-paint/pkg/paint.js';

let wasmReadyPromise = null;
let universe = null;
let canvas = null;
let gl = null;
let program = null;
let vao = null;
let texture = null;
let surfaceWidth = 0;
let surfaceHeight = 0;
let imageWidth = 0;
let imageHeight = 0;
let wasm;
let quadBuffer = null;

self.onmessage = async (event) => {
  const data = event.data;

  try {
    switch (data.type) {
      case 'test':
        break;

      case 'init':
        console.log('canvas', data.canvas);
        wasm = await init();
        await initialize(
          data.canvas,
          data.width,
          data.height,
          data.devicePixelRatio ?? 1
        );
        self.postMessage({ type: 'ready' });
        break;

      case 'resize':
        resizeSurface(data.width, data.height, data.devicePixelRatio ?? 1);
        updateQuad();
        render();
        break;

      case 'clear':
        ensureUniverse();
        universe.clear();
        imageWidth = universe.getWidth();
        imageHeight = universe.getHeight();
        updateQuad();
        uploadTexture(readPixels());
        render();
        self.postMessage({ type: 'cleared' });
        break;

      case 'decode-url': {
        const response = await fetch(data.url);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        const buffer = await response.arrayBuffer();
        await decodeAndRender(new Uint8Array(buffer));
        break;
      }

      case 'decode-file': {
        const buffer = await data.file.arrayBuffer();
        await decodeAndRender(new Uint8Array(buffer));
        break;
      }

      default:
        self.postMessage({
          type: 'error',
          message: 'unknown message type',
          detail: String(data?.type ?? '(missing)'),
        });
        break;
    }
  } catch (error) {
    self.postMessage({
      type: 'error',
      message: 'worker failed',
      detail: error instanceof Error ? error.message : String(error),
    });
  }
};

async function initialize(offscreenCanvas, width, height, devicePixelRatio) {
  canvas = offscreenCanvas;
  surfaceWidth = width;
  surfaceHeight = height;

  gl = canvas.getContext('webgl2', {
    alpha: false,
    antialias: false,
    depth: false,
    stencil: false,
    preserveDrawingBuffer: false,
  });

  if (!gl) {
    throw new Error('WebGL2 is not available in this worker');
  }

  await ensureWasmReady();
  setupGl();

  universe = new Universe(Math.max(1, width), Math.max(1, height));
  imageWidth = universe.getWidth();
  imageHeight = universe.getHeight();

  resizeSurface(width, height, devicePixelRatio);
  updateQuad();
  uploadTexture(readPixels());
  render();
}

async function ensureWasmReady() {
  if (!wasmReadyPromise) {
    wasmReadyPromise = init();
  }
  await wasmReadyPromise;
}

function ensureUniverse() {
  if (!universe) {
    throw new Error('Universe is not initialized');
  }
}

async function decodeAndRender(buffer) {
  ensureUniverse();

  const startedAt = performance.now();

  universe = new Universe(0, 0);
  universe.imageDecoder(buffer, 0);
  imageWidth = universe.getWidth();
  imageHeight = universe.getHeight();

  syncFrameToTexture();
  start_draw();

  self.postMessage({
    type: 'decoded',
    width: imageWidth,
    height: imageHeight,
    decodeMs: performance.now() - startedAt,
  });
}

function updateQuad() {
  if (!gl || !quadBuffer || !canvas) {
    return;
  }

  const safeImageWidth = Math.max(1, imageWidth);
  const safeImageHeight = Math.max(1, imageHeight);
  const safeCanvasWidth = Math.max(1, canvas.width);
  const safeCanvasHeight = Math.max(1, canvas.height);

  const scale = Math.min(
    safeCanvasWidth / safeImageWidth,
    safeCanvasHeight / safeImageHeight
  );

  const sx = (safeImageWidth * scale) / safeCanvasWidth;
  const sy = (safeImageHeight * scale) / safeCanvasHeight;

  const vertices = new Float32Array([
    -sx, -sy,
     sx, -sy,
    -sx,  sy,

    -sx,  sy,
     sx, -sy,
     sx,  sy,
  ]);

  gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);
}

function readPixels() {
  ensureUniverse();
  const ptr = universe.getBuffer();
  const width = universe.getWidth();
  const height = universe.getHeight();
  const len = width * height * 4;
  return new Uint8Array(wasm.memory.buffer, ptr, len);
}

function resizeSurface(width, height, devicePixelRatio) {
  surfaceWidth = width;
  surfaceHeight = height;
  canvas.width = Math.max(1, Math.floor(width * devicePixelRatio));
  canvas.height = Math.max(1, Math.floor(height * devicePixelRatio));
}

function syncFrameToTexture() {
  universe.combine();
  imageWidth = universe.getWidth();
  imageHeight = universe.getHeight();
  updateQuad();
  uploadTexture(readPixels());
  render();
}

function setupGl() {
  const vertexSource = `#version 300 es
    precision mediump float;
    in vec2 aPos;
    out vec2 vUv;

    void main() {
      gl_Position = vec4(aPos, 0.0, 1.0);
      vUv = vec2(
        (aPos.x + 1.0) * 0.5,
        (aPos.y + 1.0) * 0.5
      );
    }
  `;

  const fragmentSource = `#version 300 es
    precision mediump float;
    uniform sampler2D uTexture;
    in vec2 vUv;
    out vec4 outColor;

    void main() {
      outColor = texture(uTexture, vec2(vUv.x, 1.0 - vUv.y));
    }
  `;

  program = createProgram(gl, vertexSource, fragmentSource);
  vao = gl.createVertexArray();
  texture = gl.createTexture();
  quadBuffer = gl.createBuffer();

  gl.bindVertexArray(vao);

  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  gl.useProgram(program);
  const uniformLocation = gl.getUniformLocation(program, 'uTexture');
  gl.uniform1i(uniformLocation, 0);

  gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([
      -1, -1,
       1, -1,
      -1,  1,

      -1,  1,
       1, -1,
       1,  1,
    ]),
    gl.DYNAMIC_DRAW
  );

  const loc = gl.getAttribLocation(program, 'aPos');
  if (loc < 0) {
    throw new Error('attribute aPos not found');
  }

  gl.enableVertexAttribArray(loc);
  gl.vertexAttribPointer(
    loc,
    2,
    gl.FLOAT,
    false,
    0,
    0
  );

  gl.bindVertexArray(null);
  gl.bindBuffer(gl.ARRAY_BUFFER, null);
}

function uploadTexture(pixels) {
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    imageWidth,
    imageHeight,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    pixels
  );
}

function render() {
  gl.viewport(0, 0, canvas.width, canvas.height);

  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);

  gl.useProgram(program);
  gl.bindVertexArray(vao);
  gl.drawArrays(gl.TRIANGLES, 0, 6);
  gl.bindVertexArray(null);
}

function createProgram(gl, vertexSource, fragmentSource) {
  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);

  const program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(program) || 'unknown link error';
    gl.deleteProgram(program);
    gl.deleteShader(vertexShader);
    gl.deleteShader(fragmentShader);
    throw new Error(`program link failed: ${log}`);
  }

  gl.deleteShader(vertexShader);
  gl.deleteShader(fragmentShader);
  return program;
}

function compileShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader) || 'unknown compile error';
    gl.deleteShader(shader);
    throw new Error(`shader compile failed: ${log}`);
  }

  return shader;
}

function start_draw() {
  if (universe.isAnimation()) {
    setTimeout(function(){draw();},120/1000);
  }
}


function draw() {
  let wait = universe.nextFrame();
  syncFrameToTexture();
  if (wait <= 10) {wait = 0.1}
  if (universe.isAnimation()) {
    setTimeout(function(){draw();},wait*1000);
  }
}
