# 注意
FireFoxではブラウザの制限でほとんど動きません。

 https://github.com/rustwasm/wasm-bindgen/issues/2549


# WebAssembly Test

Color model ABGR uint32LE

2022/02/20 0.0.1 Pointのみ

2022/02/21 0.0.2 line追加

2022/02/21 0.0.3 rect追加

2022/02/22 0.0.4 polygram追加

2022/02/22 0.0.5 paint/picker追加

2022/02/27 0.0.6 circle/ellipse/arc追加

2022/03/07 0.0.7 jpeg decoder(baselineのみ)追加

2022/03/12 0.0.8 callback systemの変更、jpegの高速化(AANアルゴリズム)

2022/03/13 0.0.9 アフィン変換

2022/03/13 0.0.10 アフィン変換 + 補完アルゴリズム（ニアレストネイバー、バイリニア、バイキュービック、Lanczos

2022/03/17 0.0.11 Graphic Loaderを分離(WML2)

2022/03/26 0.0.12 ベジェ曲線を実装

2022/03/26 0.0.13 Screen Traitを実装

2022/03/27 0.0.14 Canvasの実装変更、Layer実装、with_alpha function実装

2022/03/29 0.0.15 JavaScriptとのバインド関数の変更、input_bufferの削除

2022/03/31 0.0.16 antialias実装(Circle以外)、Layer拡張、Layer combined canvas

2022/04/07 0.0.17 antialias circle/ellipse/arc

2022/04/08 0.0.18 resized image loader/Affine逆変換の計算式のシフトの部分が間違っていたので修正

2022/04/24 0.0.19 WML2 0.0.10で追加されたmetadata を追加

2022/05/01 0.0.20 Animation を追加

2022/05/22 0.0.21 filterを追加

2022/05/25 0.0.22 paintcore分離

Todo
- organize Functions
- border
- polyline
- timer interrupt 1/30s 1/60s 1/120s
- triming
- image saver
- inclued Trait on Canvas
- line width
- line dash
- gradient paint
- shadow effects
- pen draw with sroke
- text draw
- font support
- image effects
- multi thread
- support SIMD/OpenCL/WebGL

- and documents
