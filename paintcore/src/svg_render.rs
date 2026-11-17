//! Deliberately bounded SVG-to-paintcore adapter. No external resources, CSS,
//! scripting, filters or silent best-effort rendering. Parsing finishes before
//! callers may draw any resulting layers. This module defines no public format.
use crate::path::*;
use roxmltree::Node;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

type Result<T> = std::result::Result<T, Error>;
type Matrix = [f32; 6];
const ID: Matrix = [1., 0., 0., 1., 0., 0.];
fn unsupported(s: &str) -> Error {
    Error::new(ErrorKind::Unsupported, format!("SVG: {s}"))
}
fn number(s: &str) -> Result<f32> {
    let n: f32 = s
        .trim()
        .parse()
        .map_err(|_| unsupported("invalid number or unsupported unit"))?;
    if n.is_finite() && n.abs() <= 1e7 {
        Ok(n)
    } else {
        Err(unsupported("nonfinite or excessive coordinate"))
    }
}
fn mul(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
fn point(m: Matrix, p: (f32, f32)) -> (f32, f32) {
    (
        m[0] * p.0 + m[2] * p.1 + m[4],
        m[1] * p.0 + m[3] * p.1 + m[5],
    )
}
fn transform(s: &str) -> Result<Matrix> {
    use svgtypes::TransformListToken::*;
    let mut out = ID;
    for t in svgtypes::TransformListParser::from(s) {
        let m = match t.map_err(|_| unsupported("invalid transform"))? {
            Matrix { a, b, c, d, e, f } => {
                [a as f32, b as f32, c as f32, d as f32, e as f32, f as f32]
            }
            Translate { tx, ty } => [1., 0., 0., 1., tx as f32, ty as f32],
            Scale { sx, sy } => [sx as f32, 0., 0., sy as f32, 0., 0.],
            Rotate { angle } => {
                let r = (angle as f32).to_radians();
                [r.cos(), r.sin(), -r.sin(), r.cos(), 0., 0.]
            }
            SkewX { angle } => [1., 0., (angle as f32).to_radians().tan(), 1., 0., 0.],
            SkewY { angle } => [1., (angle as f32).to_radians().tan(), 0., 1., 0., 0.],
        };
        out = mul(out, m);
        if !out.iter().all(|n| n.is_finite() && n.abs() <= 1e7) {
            return Err(unsupported("excessive transform"));
        }
    }
    Ok(out)
}
fn color(s: &str) -> Result<u32> {
    let c: svgtypes::Color = s.parse().map_err(|_| unsupported("unsupported color"))?;
    Ok(((c.alpha as u32) << 24) | ((c.red as u32) << 16) | ((c.green as u32) << 8) | c.blue as u32)
}
fn props<'a, 'i>(n: Node<'a, 'i>) -> Result<HashMap<&'a str, &'a str>> {
    let mut p = HashMap::new();
    for a in n.attributes() {
        let k = a.name();
        if a.namespace() == Some("http://www.w3.org/XML/1998/namespace")
            && k == "space"
            && matches!(a.value(), "preserve" | "default")
        {
            continue;
        }
        if n.tag_name().name() == "svg"
            && k == "version"
            && matches!(a.value(), "1.0" | "1.1" | "2.0")
        {
            continue;
        }
        if a.namespace().is_some()
            && !(a.namespace() == Some("http://www.w3.org/1999/xlink")
                && k == "href"
                && matches!(n.tag_name().name(), "linearGradient" | "radialGradient"))
        {
            return Err(unsupported("namespaced attribute"));
        }
        let shared = matches!(k, "id" | "data-name");
        let graphics = matches!(
            k,
            "transform"
                | "fill"
                | "fill-rule"
                | "stroke"
                | "stroke-width"
                | "stroke-linecap"
                | "stroke-linejoin"
                | "stroke-miterlimit"
                | "style"
        );
        let gradient = matches!(
            k,
            "gradientUnits" | "gradientTransform" | "spreadMethod" | "href"
        );
        let allowed = shared
            || match n.tag_name().name() {
                "svg" => {
                    graphics
                        || matches!(
                            k,
                            "viewBox"
                                | "width"
                                | "height"
                                | "overflow"
                                | "preserveAspectRatio"
                                | "x"
                                | "y"
                        )
                }
                "g" => graphics,
                "path" => graphics || k == "d",
                "circle" => graphics || matches!(k, "cx" | "cy" | "r"),
                "ellipse" => graphics || matches!(k, "cx" | "cy" | "rx" | "ry"),
                "rect" => graphics || matches!(k, "x" | "y" | "width" | "height" | "rx" | "ry"),
                "line" => graphics || matches!(k, "x1" | "y1" | "x2" | "y2"),
                "polyline" | "polygon" => graphics || k == "points",
                "defs" => false,
                "radialGradient" => gradient || matches!(k, "cx" | "cy" | "r" | "fx" | "fy"),
                "linearGradient" => gradient || matches!(k, "x1" | "y1" | "x2" | "y2"),
                "stop" => matches!(k, "offset" | "stop-color" | "stop-opacity" | "style"),
                _ => false,
            };
        if !allowed {
            return Err(unsupported(&format!(
                "attribute {k} on {}",
                n.tag_name().name()
            )));
        }
        if k == "href" && p.contains_key(k) {
            return Err(unsupported("ambiguous gradient href"));
        }
        p.insert(k, a.value());
    }
    if let Some(style) = n.attribute("style") {
        for pair in style.split(';').filter(|s| !s.trim().is_empty()) {
            let (k, v) = pair
                .split_once(':')
                .ok_or_else(|| unsupported("malformed style"))?;
            let k = k.trim();
            // Background capture affects filter compositing only. Filters and text
            // are unsupported, so these producer hints cannot affect this subset.
            if k == "enable-background" {
                let tokens: Vec<_> = v.split_whitespace().collect();
                if tokens.first() == Some(&"new")
                    && (tokens.len() == 1
                        || (tokens.len() == 5 && tokens[1..].iter().all(|s| number(s).is_ok())))
                {
                    continue;
                }
                return Err(unsupported("enable-background value"));
            }
            if !(if n.tag_name().name() == "stop" {
                matches!(k, "stop-color" | "stop-opacity")
            } else {
                matches!(
                    k,
                    "fill"
                        | "fill-rule"
                        | "stroke"
                        | "stroke-width"
                        | "stroke-linecap"
                        | "stroke-linejoin"
                        | "stroke-miterlimit"
                )
            }) {
                return Err(unsupported(&format!("style {k}")));
            }
            p.insert(k, v.trim());
        }
    }
    Ok(p)
}
#[derive(Clone)]
struct State {
    matrix: Matrix,
    fill: String,
    rule: FillRule,
    stroke: String,
    stroke_style: StrokeStyle,
}
/// Rendering-only state, deliberately not part of the public glyph command ABI.
#[derive(Debug, Clone)]
pub(crate) struct SvgPreparedLayer {
    pub(crate) path: PathGlyphLayer,
    pub(crate) stroke_style: Option<StrokeStyle>,
}
impl std::ops::Deref for SvgPreparedLayer {
    type Target = PathGlyphLayer;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}
fn path(data: &str, m: Matrix) -> Result<Vec<Command>> {
    use svgtypes::PathSegment::*;
    let mut out = Vec::new();
    let mut cur = (0f32, 0f32);
    let mut start = cur;
    let mut cubic = None;
    let mut quad = None;
    for seg in svgtypes::PathParser::from(data) {
        if out.len() >= 100_000 {
            return Err(unsupported("path command limit"));
        }
        let seg = seg.map_err(|_| unsupported("invalid path"))?;
        let pos = |x: f64, y: f64, abs: bool| -> Result<(f32, f32)> {
            let p = (
                x as f32 + if abs { 0. } else { cur.0 },
                y as f32 + if abs { 0. } else { cur.1 },
            );
            if ![p.0, p.1].iter().all(|n| n.is_finite() && n.abs() <= 1e7) {
                return Err(unsupported("excessive path coordinate"));
            }
            Ok(p)
        };
        let old_cubic = cubic.take();
        let old_quad = quad.take();
        match seg {
            MoveTo { abs, x, y } => {
                cur = pos(x, y, abs)?;
                start = cur;
                let p = point(m, cur);
                out.push(Command::MoveTo(p.0, p.1));
            }
            LineTo { abs, x, y } => {
                cur = pos(x, y, abs)?;
                let p = point(m, cur);
                out.push(Command::Line(p.0, p.1));
            }
            HorizontalLineTo { abs, x } => {
                cur = pos(x, if abs { cur.1 as f64 } else { 0. }, abs)?;
                let p = point(m, cur);
                out.push(Command::Line(p.0, p.1));
            }
            VerticalLineTo { abs, y } => {
                cur = pos(if abs { cur.0 as f64 } else { 0. }, y, abs)?;
                let p = point(m, cur);
                out.push(Command::Line(p.0, p.1));
            }
            CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let a = pos(x1, y1, abs)?;
                let b = pos(x2, y2, abs)?;
                cur = pos(x, y, abs)?;
                cubic = Some(b);
                out.push(Command::CubicBezier(
                    point(m, a),
                    point(m, b),
                    point(m, cur),
                ));
            }
            SmoothCurveTo { abs, x2, y2, x, y } => {
                let a = old_cubic
                    .map(|p: (f32, f32)| (2. * cur.0 - p.0, 2. * cur.1 - p.1))
                    .unwrap_or(cur);
                let b = pos(x2, y2, abs)?;
                cur = pos(x, y, abs)?;
                cubic = Some(b);
                out.push(Command::CubicBezier(
                    point(m, a),
                    point(m, b),
                    point(m, cur),
                ));
            }
            Quadratic { abs, x1, y1, x, y } => {
                let a = pos(x1, y1, abs)?;
                cur = pos(x, y, abs)?;
                quad = Some(a);
                out.push(Command::Bezier(point(m, a), point(m, cur)));
            }
            SmoothQuadratic { abs, x, y } => {
                let a = old_quad
                    .map(|p: (f32, f32)| (2. * cur.0 - p.0, 2. * cur.1 - p.1))
                    .unwrap_or(cur);
                cur = pos(x, y, abs)?;
                quad = Some(a);
                out.push(Command::Bezier(point(m, a), point(m, cur)));
            }
            ClosePath { .. } => {
                cur = start;
                out.push(Command::Close);
            }
            EllipticalArc { .. } => return Err(unsupported("elliptical arcs are not implemented")),
        }
    }
    for command in &out {
        let valid = |p: (f32, f32)| {
            p.0.is_finite() && p.1.is_finite() && p.0.abs() <= 1e7 && p.1.abs() <= 1e7
        };
        let bounded = match command {
            Command::MoveTo(x, y) | Command::Line(x, y) => valid((*x, *y)),
            Command::Bezier(a, b) => valid(*a) && valid(*b),
            Command::CubicBezier(a, b, c) => valid(*a) && valid(*b) && valid(*c),
            Command::Close => true,
        };
        if !bounded {
            return Err(unsupported("transformed path coordinate limit"));
        }
    }
    if !out.is_empty() && !matches!(out.first(), Some(Command::MoveTo(..))) {
        return Err(unsupported("path must begin with moveto"));
    }
    Ok(out)
}

fn shape(tag: &str, p: &HashMap<&str, &str>, m: Matrix) -> Result<Vec<Command>> {
    let get = |k: &str, default: f32| p.get(k).map(|v| number(v)).unwrap_or(Ok(default));
    if tag == "path" {
        return path(p.get("d").copied().unwrap_or(""), m);
    }
    let data = match tag {
        "line" => format!(
            "M{} {} L{} {}",
            get("x1", 0.)?,
            get("y1", 0.)?,
            get("x2", 0.)?,
            get("y2", 0.)?
        ),
        "rect" => {
            let x = get("x", 0.)?;
            let y = get("y", 0.)?;
            let w = get("width", 0.)?;
            let h = get("height", 0.)?;
            if w < 0. || h < 0. {
                return Err(unsupported("negative rectangle dimensions"));
            }
            if get("rx", 0.)? != 0. || get("ry", 0.)? != 0. {
                return Err(unsupported("rounded rectangles"));
            }
            if w == 0. || h == 0. {
                return Ok(Vec::new());
            }
            format!("M{x} {y} h{w} v{h} h{} Z", -w)
        }
        "circle" | "ellipse" => {
            let cx = get("cx", 0.)?;
            let cy = get("cy", 0.)?;
            let rx = if tag == "circle" {
                get("r", 0.)?
            } else {
                get("rx", 0.)?
            };
            let ry = if tag == "circle" { rx } else { get("ry", 0.)? };
            if rx < 0. || ry < 0. {
                return Err(unsupported("negative ellipse radius"));
            }
            if rx == 0. || ry == 0. {
                return Ok(Vec::new());
            }
            let k = 0.5522848;
            let dx = k * rx;
            let dy = k * ry;
            format!("M{} {cy} C{} {} {} {} {cx} {} C{} {} {} {} {} {cy} C{} {} {} {} {cx} {} C{} {} {} {} {} {cy} Z",cx+rx,cx+rx,cy+dy,cx+dx,cy+ry,cy+ry,cx-dx,cy+ry,cx-rx,cy+dy,cx-rx,cx-rx,cy-dy,cx-dx,cy-ry,cy-ry,cx+dx,cy-ry,cx+rx,cy-dy,cx+rx)
        }
        "polyline" | "polygon" => {
            let values: Vec<_> =
                svgtypes::NumberListParser::from(p.get("points").copied().unwrap_or(""))
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|_| unsupported("invalid polygon points"))?;
            if values.len() % 2 != 0 || values.len() > 200_000 {
                return Err(unsupported("invalid or excessive polygon points"));
            }
            if values.is_empty() {
                return Ok(Vec::new());
            }
            let mut d = String::new();
            for (i, p) in values.chunks_exact(2).enumerate() {
                d.push_str(&format!(
                    "{}{} {} ",
                    if i == 0 { "M" } else { "L" },
                    p[0],
                    p[1]
                ));
            }
            if tag == "polygon" {
                d.push('Z');
            }
            d
        }
        _ => return Err(unsupported("shape")),
    };
    path(&data, m)
}

fn gradient<'a, 'i>(
    n: Node<'a, 'i>,
    m: Matrix,
    ids: &HashMap<String, Node<'a, 'i>>,
) -> Result<GlyphPaint> {
    let mut chain = Vec::new();
    let mut current = n;
    loop {
        if chain.len() >= 32 || chain.iter().any(|v: &Node<'a, 'i>| v.id() == current.id()) {
            return Err(unsupported("cyclic or deep gradient inheritance"));
        }
        if !matches!(
            current.tag_name().name(),
            "linearGradient" | "radialGradient"
        ) {
            return Err(unsupported("gradient href target"));
        }
        if current
            .children()
            .filter(|n| n.is_element())
            .any(|n| n.tag_name().name() != "stop")
        {
            return Err(unsupported("gradient child"));
        }
        chain.push(current);
        let p = props(current)?;
        match p.get("href") {
            Some(href) => {
                let id = href
                    .strip_prefix('#')
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| unsupported("external gradient href"))?;
                current = *ids
                    .get(id)
                    .ok_or_else(|| unsupported("missing gradient href"))?;
            }
            None => break,
        }
    }
    let mut p = HashMap::new();
    for node in chain.iter().rev() {
        p.extend(props(*node)?);
    }
    let stops_node = chain
        .iter()
        .find(|node| node.children().any(|n| n.is_element()))
        .copied();
    if p.get("gradientUnits").copied() != Some("userSpaceOnUse") {
        return Err(unsupported("only userSpaceOnUse gradients are implemented"));
    }
    let get = |k: &str, default: f32| -> Result<f32> {
        p.get(k).map(|v| number(v)).unwrap_or(Ok(default))
    };
    let mut stops = Vec::new();
    for child in stops_node
        .into_iter()
        .flat_map(|node| node.children())
        .filter(|n| n.is_element())
    {
        if child.tag_name().name() != "stop" {
            return Err(unsupported("gradient child"));
        }
        if child.children().any(|n| n.is_element()) {
            return Err(unsupported("stop children"));
        }
        let s = props(child)?;
        let val = s.get("offset").copied().unwrap_or("0");
        let offset = if let Some(v) = val.strip_suffix('%') {
            number(v)? / 100.
        } else {
            number(val)?
        };
        let offset = offset
            .clamp(0., 1.)
            .max(stops.last().map(|s: &GradientStop| s.offset).unwrap_or(0.));
        let opacity = match s.get("stop-opacity") {
            Some(v) => {
                if let Some(v) = v.strip_suffix('%') {
                    number(v)? / 100.
                } else {
                    number(v)?
                }
            }
            None => 1.,
        }
        .clamp(0., 1.);
        let value = color(s.get("stop-color").copied().unwrap_or("black"))?;
        let alpha = (((value >> 24) as f32) * opacity).round() as u32;
        stops.push(GradientStop {
            offset,
            color: (value & 0x00ff_ffff) | (alpha << 24),
        });
    }
    if stops.is_empty() {
        return Err(unsupported("empty gradient"));
    }
    let transform = mul(
        m,
        transform(p.get("gradientTransform").copied().unwrap_or(""))?,
    );
    let determinant = transform[0] * transform[3] - transform[1] * transform[2];
    if !transform.iter().all(|n| n.is_finite() && n.abs() <= 1e7)
        || !determinant.is_finite()
        || determinant.abs() <= f32::EPSILON
    {
        return Err(unsupported("singular or excessive gradient transform"));
    }
    let spread = match p.get("spreadMethod").copied().unwrap_or("pad") {
        "pad" => GradientSpread::Pad,
        "reflect" => GradientSpread::Reflect,
        "repeat" => GradientSpread::Repeat,
        _ => return Err(unsupported("gradient spread")),
    };
    let units = GradientUnits::UserSpaceOnUse;
    match n.tag_name().name() {
        "radialGradient" => {
            if !["cx", "cy", "r"].iter().all(|k| p.contains_key(k)) {
                return Err(unsupported("radial gradient percentage defaults"));
            }
            let cx = get("cx", 0.)?;
            let cy = get("cy", 0.)?;
            let r = get("r", 0.)?;
            if r <= 0. {
                return Err(unsupported("nonpositive gradient radius"));
            }
            let fx = get("fx", cx)?;
            let fy = get("fy", cy)?;
            if (fx - cx).hypot(fy - cy) >= r {
                return Err(unsupported("radial focus outside circle"));
            }
            Ok(GlyphPaint::RadialGradient(RadialGradientPaint {
                cx,
                cy,
                r,
                fx,
                fy,
                fr: 0.,
                units,
                transform,
                spread,
                stops,
            }))
        }
        "linearGradient" => {
            if !["x1", "y1", "x2", "y2"].iter().all(|k| p.contains_key(k)) {
                return Err(unsupported("linear gradient percentage defaults"));
            }
            Ok(GlyphPaint::LinearGradient(LinearGradientPaint {
                x1: get("x1", 0.)?,
                y1: get("y1", 0.)?,
                x2: get("x2", 1.)?,
                y2: get("y2", 0.)?,
                units,
                transform,
                spread,
                stops,
            }))
        }
        _ => Err(unsupported("paint reference is not a gradient")),
    }
}
fn visit(
    n: Node<'_, '_>,
    state: &State,
    ids: &HashMap<String, Node<'_, '_>>,
    out: &mut Vec<SvgPreparedLayer>,
    depth: usize,
) -> Result<()> {
    if depth > 128 {
        return Err(unsupported("element nesting limit"));
    }
    let tag = n.tag_name().name();
    if !matches!(
        tag,
        "svg"
            | "g"
            | "path"
            | "circle"
            | "ellipse"
            | "rect"
            | "line"
            | "polyline"
            | "polygon"
            | "defs"
            | "title"
            | "desc"
            | "linearGradient"
            | "radialGradient"
    ) {
        return Err(unsupported(&format!("element {tag}")));
    }
    if matches!(tag, "title" | "desc") {
        return Ok(());
    }
    let p = props(n)?;
    if matches!(tag, "defs" | "linearGradient" | "radialGradient") {
        return Ok(());
    }
    if tag == "svg" && depth != 0 {
        return Err(unsupported("nested svg viewport"));
    }
    if tag == "svg"
        && ["x", "y"]
            .iter()
            .any(|k| p.get(k).is_some_and(|s| !matches!(*s, "0" | "0px")))
    {
        return Err(unsupported("SVG viewport position"));
    }
    let mut state = state.clone();
    state.matrix = mul(
        state.matrix,
        transform(p.get("transform").copied().unwrap_or(""))?,
    );
    if !state.matrix.iter().all(|n| n.is_finite() && n.abs() <= 1e7) {
        return Err(unsupported("cumulative transform limit"));
    }
    if let Some(fill) = p.get("fill") {
        state.fill = fill.to_string();
    }
    if let Some(stroke) = p.get("stroke") {
        state.stroke = stroke.to_string();
    }
    if let Some(v) = p.get("stroke-width") {
        state.stroke_style.width = number(v)?;
        if state.stroke_style.width < 0. {
            return Err(unsupported("negative stroke width"));
        }
    }
    if let Some(v) = p.get("stroke-linecap") {
        state.stroke_style.cap = match *v {
            "butt" => StrokeCap::Butt,
            "round" => StrokeCap::Round,
            "square" => StrokeCap::Square,
            _ => return Err(unsupported("stroke-linecap")),
        };
    }
    if let Some(v) = p.get("stroke-linejoin") {
        state.stroke_style.join = match *v {
            "miter" => StrokeJoin::Miter,
            "round" => StrokeJoin::Round,
            "bevel" => StrokeJoin::Bevel,
            _ => return Err(unsupported("stroke-linejoin")),
        };
    }
    if let Some(v) = p.get("stroke-miterlimit") {
        state.stroke_style.miter_limit = number(v)?;
        if state.stroke_style.miter_limit < 1. {
            return Err(unsupported("stroke-miterlimit below one"));
        }
    }

    if let Some(rule) = p.get("fill-rule") {
        state.rule = match *rule {
            "nonzero" => FillRule::NonZero,
            "evenodd" => FillRule::EvenOdd,
            _ => return Err(unsupported("fill-rule")),
        };
    }
    let geometry = matches!(
        tag,
        "path" | "circle" | "ellipse" | "rect" | "line" | "polyline" | "polygon"
    );
    if geometry
        && n.children()
            .filter(|n| n.is_element())
            .any(|n| !matches!(n.tag_name().name(), "title" | "desc"))
    {
        return Err(unsupported("rendering children inside path"));
    }
    if geometry {
        let commands = shape(tag, &p, state.matrix)?;
        let paint_for = |value: &str| -> Result<GlyphPaint> {
            let value = value.trim();
            if value == "currentColor" {
                Ok(GlyphPaint::CurrentColor)
            } else if let Some(id) = value
                .strip_prefix("url(#")
                .and_then(|s| s.strip_suffix(')'))
            {
                gradient(
                    *ids.get(id).ok_or_else(|| unsupported("missing gradient"))?,
                    state.matrix,
                    ids,
                )
            } else {
                Ok(GlyphPaint::Solid(color(value)?))
            }
        };
        let mut add = |path: PathGlyphLayer, stroke_style: Option<StrokeStyle>| -> Result<()> {
            if out.len() >= 4096 {
                return Err(unsupported("painted layer limit"));
            }
            let total: usize = out.iter().map(|l| l.path.commands.len()).sum();
            if total + path.commands.len() > 100_000 {
                return Err(unsupported("total path command limit"));
            }
            out.push(SvgPreparedLayer { path, stroke_style });
            Ok(())
        };
        if !commands.is_empty() {
            if state.fill != "none" && tag != "line" {
                let mut layer = PathGlyphLayer::new(commands.clone(), paint_for(&state.fill)?);
                layer.fill_rule = state.rule;
                add(layer, None)?;
            }
            if state.stroke != "none" && state.stroke_style.width > 0. {
                let m = state.matrix;
                let sx = m[0].hypot(m[1]);
                let sy = m[2].hypot(m[3]);
                let dot = m[0] * m[2] + m[1] * m[3];
                if sx <= f32::EPSILON
                    || (sx - sy).abs() > 1e-5 * sx.max(sy)
                    || dot.abs() > 1e-5 * sx * sy
                {
                    return Err(unsupported("nonuniform or skewed stroke transform"));
                }
                let mut style = state.stroke_style.clone();
                style.width *= sx;
                if !style.width.is_finite() || style.width > 1e7 {
                    return Err(unsupported("excessive transformed stroke width"));
                }
                let layer =
                    PathGlyphLayer::stroke(commands, paint_for(&state.stroke)?, style.width);
                add(layer, Some(style))?;
            }
        }
    }
    for child in n.children().filter(|n| n.is_element()) {
        visit(child, &state, ids, out, depth + 1)?;
    }
    Ok(())
}
/// Convert a validated, bounded subset to existing paintcore primitives.
pub(crate) fn parse_document(
    document: &str,
    units_to_pixels: [f32; 6],
) -> Result<Vec<SvgPreparedLayer>> {
    if document.len() > 4 * 1024 * 1024 {
        return Err(unsupported("document size limit"));
    }
    let doc = roxmltree::Document::parse_with_options(
        document,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 100_000,
        },
    )
    .map_err(|_| unsupported("invalid XML"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(unsupported("root must be svg"));
    }
    let mut ids = HashMap::new();
    let mut count = 0;
    for n in doc.descendants().filter(|n| n.is_element()) {
        count += 1;
        if matches!(
            n.tag_name().name(),
            "style" | "script" | "animate" | "animateTransform" | "set"
        ) {
            return Err(unsupported(
                "CSS, scripting and animation are not implemented",
            ));
        }
        if count > 100_000 {
            return Err(unsupported("element count limit"));
        }
        if n.tag_name()
            .namespace()
            .is_some_and(|s| s != "http://www.w3.org/2000/svg")
        {
            return Err(unsupported("foreign element namespace"));
        }
        if let Some(id) = n.attribute("id") {
            if ids.insert(id.to_owned(), n).is_some() {
                return Err(unsupported("duplicate id"));
            }
        }
    }
    let view: Vec<f32> = if let Some(v) = root.attribute("viewBox") {
        v.split(|c: char| c.is_ascii_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(number)
            .collect::<Result<_>>()?
    } else {
        vec![
            0.,
            0.,
            number(
                root.attribute("width")
                    .ok_or_else(|| unsupported("missing SVG width"))?,
            )?,
            number(
                root.attribute("height")
                    .ok_or_else(|| unsupported("missing SVG height"))?,
            )?,
        ]
    };
    if view.len() != 4 || view[2] <= 0. || view[3] <= 0. {
        return Err(unsupported("invalid viewBox"));
    }
    if root
        .attribute("preserveAspectRatio")
        .is_some_and(|s| s != "none" && s != "xMidYMid meet")
    {
        return Err(unsupported("preserveAspectRatio"));
    }
    if root.attribute("overflow").is_some_and(|s| s != "visible") {
        return Err(unsupported("viewport clipping"));
    }
    let width = root
        .attribute("width")
        .map(number)
        .transpose()?
        .unwrap_or(view[2]);
    let height = root
        .attribute("height")
        .map(number)
        .transpose()?
        .unwrap_or(view[3]);
    if width <= 0.
        || height <= 0.
        || !units_to_pixels
            .iter()
            .all(|n| n.is_finite() && n.abs() <= 1e7)
    {
        return Err(unsupported("invalid explicit SVG transform or viewport"));
    }
    let mut sx = width / view[2];
    let mut sy = height / view[3];
    let mut dx = 0.;
    let mut dy = 0.;
    if root.attribute("preserveAspectRatio") != Some("none") {
        let scale = sx.min(sy);
        sx = scale;
        sy = scale;
        dx = (width - view[2] * scale) * 0.5;
        dy = (height - view[3] * scale) * 0.5;
    }
    let viewport = [sx, 0., 0., sy, dx - view[0] * sx, dy - view[1] * sy];
    let matrix = mul(units_to_pixels, viewport);
    let state = State {
        matrix,
        fill: "black".into(),
        rule: FillRule::NonZero,
        stroke: "none".into(),
        stroke_style: StrokeStyle::default(),
    };
    let mut out = Vec::new();
    visit(root, &state, &ids, &mut out, 0)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(layer: &SvgGlyphLayer) -> Result<Vec<SvgPreparedLayer>> {
        parse_document(&layer.document, [2., 0., 0., 2., 0., 0.])
    }
    fn svg(body: &str) -> SvgGlyphLayer {
        SvgGlyphLayer {
            document: format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'>{body}</svg>"
            ),
            view_box_min_x: 0.,
            view_box_min_y: 0.,
            view_box_width: 20.,
            view_box_height: 20.,
            width: 20.,
            height: 20.,
            offset_x: 0.,
            offset_y: 0.,
        }
    }
    #[test]
    fn relative_and_smooth_controls_are_expanded_before_scaling() {
        let layers = parse(&svg("<path d='M1 1 c1 0 2 1 3 1 s2 1 3 0'/>")).unwrap();
        assert!(matches!(
            layers[0].commands[2],
            Command::CubicBezier((10., 4.), (12., 6.), (14., 4.))
        ));
    }
    #[test]
    fn quadratic_reflection_resets_after_line() {
        let layers = parse(&svg("<path d='M1 1 q1 2 2 0 t2 0 l1 0 t1 0'/>")).unwrap();
        assert!(matches!(
            layers[0].commands[2],
            Command::Bezier((8., -2.), (10., 2.))
        ));
        assert!(matches!(
            layers[0].commands[4],
            Command::Bezier((12., 2.), (14., 2.))
        ));
    }
    #[test]
    fn unsupported_content_in_defs_cannot_change_style_silently() {
        assert!(parse(&svg(
            "<defs><style>path { fill: red }</style></defs><path d='M0 0 L1 1'/>"
        ))
        .is_err());
    }
    #[test]
    fn gradients_and_viewports_reject_unsupported_defaults_and_positions() {
        assert!(parse(&svg("<defs><radialGradient id='r' gradientUnits='userSpaceOnUse'><stop stop-color='red'/></radialGradient></defs><path fill='url(#r)' d='M0 0 L1 1'/>")).is_err());
        let mut layer = svg("<path d='M0 0 L1 1'/>");
        layer.document = layer.document.replace("viewBox=", "x='1' viewBox=");
        assert!(parse(&layer).is_err());
    }
    #[test]
    fn basic_shapes_preserve_closure_and_scaled_stroke_style() {
        let layers=parse(&svg("<circle cx='2' cy='2' r='1'/><ellipse cx='4' cy='4' rx='2' ry='1'/><rect x='1' y='1' width='2' height='3'/><polygon points='1,1 2,1 2,2'/><polyline fill='none' stroke='red' points='0,0 2,2'/><line x1='1' y1='2' x2='3' y2='4' stroke='blue' stroke-width='3' stroke-linecap='round' stroke-linejoin='bevel' stroke-miterlimit='10'/>")).unwrap();
        assert_eq!(layers.len(), 6);
        assert!(matches!(layers[0].commands.last(), Some(Command::Close)));
        assert!(matches!(layers[3].commands.last(), Some(Command::Close)));
        assert!(!matches!(layers[4].commands.last(), Some(Command::Close)));
        let style = layers[5].stroke_style.as_ref().unwrap();
        assert_eq!(style.width, 6.);
        assert_eq!(style.cap, StrokeCap::Round);
        assert_eq!(style.join, StrokeJoin::Bevel);
        assert_eq!(style.miter_limit, 10.);
    }
    #[test]
    fn stroke_nonuniform_transform_is_not_silently_approximated() {
        assert!(parse(&svg(
            "<path d='M0 0 L2 2' stroke='red' transform='scale(2 1)'/>"
        ))
        .is_err());
    }
    #[test]
    fn inherited_gradient_stops_keep_opacity_and_local_overrides() {
        let layers=parse(&svg("<radialGradient id='base' gradientUnits='userSpaceOnUse' cx='4' cy='4' r='3'><stop offset='0' stop-color='#ff0000' stop-opacity='.5'/><stop offset='1' stop-color='blue'/></radialGradient><radialGradient id='copy' href='#base' cx='5'/><rect width='10' height='10' fill='url(#copy)'/>")).unwrap();
        let GlyphPaint::RadialGradient(g) = &layers[0].paint else {
            panic!("expected gradient")
        };
        assert_eq!(g.cx, 5.);
        assert_eq!(g.cy, 4.);
        assert_eq!(g.fx, 5.);
        assert_eq!(g.stops[0].color, 0x80ff0000);
    }
    #[test]
    fn cyclic_and_external_gradient_references_fail() {
        for reference in ["#b", "https://example.com/gradient"] {
            let source=format!("<linearGradient id='a' href='{reference}'/><linearGradient id='b' href='#a'/><rect width='2' height='2' fill='url(#a)'/>");
            assert!(parse(&svg(&source)).is_err());
        }
    }
    #[test]
    fn known_adobe_metadata_is_accepted_but_other_styles_are_not() {
        let source="<svg xmlns='http://www.w3.org/2000/svg' version='1.1' xml:space='preserve' x='0px' y='0px' viewBox='0 0 128 128' style='enable-background:new 0 0 128 128;'><rect width='2' height='2'/></svg>";
        assert!(parse_document(source, ID).is_ok());
        assert!(parse_document(
            &source.replace("enable-background:new 0 0 128 128", "filter:url(#x)"),
            ID
        )
        .is_err());
    }
}
