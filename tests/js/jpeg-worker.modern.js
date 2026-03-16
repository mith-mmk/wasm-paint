self.postMessage({ type: 'test' });

import init, {UniverseFast} from '../../wasm-paint/pkg/paint.js';

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


self.onmessage = async (event) => {
  const data = event.data;

  try {
    switch (data.type) {
      case 'test':
        break;
      case 'init':
        console.log("canvas", data.canvas);
        wasm = await init();
        await initialize(data.canvas, data.width, data.height, data.devicePixelRatio ?? 1);
        self.postMessage({ type: 'ready' });
        break;

      case 'resize':
        resizeSurface(data.width, data.height, data.devicePixelRatio ?? 1);
        render();
        break;

      case 'clear':
        ensureUniverse();
        universe.clear();
        imageWidth = universe.width();
        imageHeight = universe.height();
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

  universe = new UniverseFast(Math.max(1, width), Math.max(1, height));
  if(!universe) {
    throw new Error('Undefine UniverseFast');
  }
  imageWidth = universe.width();
  imageHeight = universe.height();

  resizeSurface(width, height, devicePixelRatio);
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
    throw new Error('UniverseFast is not initialized');
  }
}

async function decodeAndRender(buffer) {
  ensureUniverse();

  const startedAt = performance.now();
  const ok = universe.decode(buffer);

  if (!ok) {
    throw new Error('image decode failed');
  }

  imageWidth = universe.width();
  imageHeight = universe.height();

  uploadTexture(readPixels());
  render();

  self.postMessage({
    type: 'decoded',
    width: imageWidth,
    height: imageHeight,
    decodeMs: performance.now() - startedAt,
  });
}

function readPixels() {
  ensureUniverse();

  const ptr = universe.ptr();
  const len = universe.len();
  return new Uint8Array(wasm.memory.buffer, ptr, len);
}

function resizeSurface(width, height, devicePixelRatio) {
  surfaceWidth = width;
  surfaceHeight = height;
  canvas.width = Math.max(1, Math.floor(width * devicePixelRatio));
  canvas.height = Math.max(1, Math.floor(height * devicePixelRatio));
}

function setupGl() {
  const vertexSource = `#version 300 es
    precision mediump float;
    const vec2 positions[3] = vec2[3](
      vec2(-1.0, -1.0),
      vec2( 3.0, -1.0),
      vec2(-1.0,  3.0)
    );
    out vec2 vUv;
    void main() {
      vec2 position = positions[gl_VertexID];
      vUv = position * 0.5 + 0.5;
      gl_Position = vec4(position, 0.0, 1.0);
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
    pixels,
  );
}

function render() {
  if (!gl || !program || !vao || !texture) {
    return;
  }

  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.useProgram(program);
  gl.bindVertexArray(vao);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
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
