import { Universe } from "../../wasm-paint/pkg/paint.js";

// Call after the paint component's ready promise. Each render owns its engine:
// failed shaping, preview and cancel can never modify the document or history.
export function fontTextImageData({ width, height, fontBytes, text, fontSize, color, flow, x, y, align = "left" }) {
  if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || height < 1 ||
      width > 8192 || height > 8192 || width * height > 16 * 1024 * 1024) {
    throw new Error("縦書きのキャンバスは各辺8192px以下・合計16Mピクセル以下にしてください。");
  }
  if (![x, y, fontSize].every(Number.isFinite) || fontSize <= 0) throw new Error("文字サイズ・位置が不正です。");
  if (!["vertical", "mixed", "sideways"].includes(flow)) throw new Error("未対応の文字方向です。");
  if (!["left", "center", "right"].includes(align) || !/^#[0-9a-f]{6}$/iu.test(color)) throw new Error("文字の色・揃え方が不正です。");
  if (typeof text !== "string" || text.length > 4096) throw new Error("縦書きは4096文字以内にしてください。");
  if (!fontBytes) throw new Error("縦書き用フォントファイルを選択してください。");
  const engine = new Universe(width, height);
  try {
    if (typeof engine.drawTextFlow !== "function" || typeof engine.measureTextFlow !== "function") {
      throw new Error("縦書きには font 機能付きの新しい Wasm ビルドが必要です。");
    }
    engine.loadFont(fontBytes);
    const bounds = engine.measureTextFlow(text, fontSize, flow);
    if (bounds.length !== 4) throw new Error("描画できる文字がありません。");
    if (!Array.from(bounds).every(Number.isFinite)) throw new Error("文字の描画範囲が不正です。");
    const [minX, minY, maxX, maxY] = bounds;
    const advance = maxY - minY;
    const offset = align === "center" ? advance / 2 : align === "right" ? advance : 0;
    const argb = (0xff000000 | Number.parseInt(color.slice(1), 16)) >>> 0;
    engine.drawTextFlow(text, x - minX, y - minY - offset, fontSize, argb, flow);
    const image = engine.getLayerImageData("main");
    // ImageData may retain a view into Wasm memory, which free/reallocation reuses.
    return new ImageData(new Uint8ClampedArray(image.data), image.width, image.height);
  } catch (error) {
    throw error instanceof Error ? error : new Error(String(error));
  } finally {
    engine.free();
  }
}
