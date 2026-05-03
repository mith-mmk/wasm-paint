#[cfg(feature = "font")]
use crate::canvas::Screen;
#[cfg(feature = "font")]
use crate::path::{draw_text_with_options, FontOptions, GlyphRun};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathScript {
    Normal,
    Superscript,
    Subscript,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathToken {
    pub text: String,
    pub script: MathScript,
}

pub fn parse_tex_like(input: &str) -> Vec<MathToken> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        let script = match ch {
            '^' => MathScript::Superscript,
            '_' => MathScript::Subscript,
            _ => {
                tokens.push(MathToken {
                    text: ch.to_string(),
                    script: MathScript::Normal,
                });
                continue;
            }
        };

        let text = if chars.peek() == Some(&'{') {
            chars.next();
            let mut text = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                text.push(ch);
            }
            text
        } else {
            chars.next().map(|ch| ch.to_string()).unwrap_or_default()
        };

        if !text.is_empty() {
            tokens.push(MathToken { text, script });
        }
    }
    tokens
}

pub fn tex_like_plain_text(input: &str) -> String {
    parse_tex_like(input)
        .into_iter()
        .map(|token| token.text)
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(feature = "font")]
pub fn draw_tex_like(
    screen: &mut dyn Screen,
    input: &str,
    mut options: FontOptions<'_>,
    x: f32,
    y: f32,
    color: u32,
) -> Result<GlyphRun, Box<dyn std::error::Error>> {
    let base_size = options.font_size;
    let script_size = base_size * 0.7;
    let mut cursor_x = x;
    let mut combined = GlyphRun::default();

    for token in parse_tex_like(input) {
        let (font_size, dy) = match token.script {
            MathScript::Normal => (base_size, 0.0),
            MathScript::Superscript => (script_size, -base_size * 0.45),
            MathScript::Subscript => (script_size, base_size * 0.25),
        };
        options.font_size = font_size;
        let run =
            draw_text_with_options(screen, &token.text, options.clone(), cursor_x, y + dy, color)?;
        let advance = run
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.glyph.metrics.advance_x)
            .fold(0.0f32, f32::max);
        cursor_x += advance;
        combined.glyphs.extend(run.glyphs);
    }

    Ok(combined)
}
