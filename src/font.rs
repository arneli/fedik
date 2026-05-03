use ratatui::style::{Color::*, Modifier, Style};
use std::{collections::HashSet, convert::TryFrom, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedTextError {
    InvalidCharacter(char),
}

impl fmt::Display for RenderedTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "invalid character: {c}"),
        }
    }
}

impl std::error::Error for RenderedTextError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedText {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub struct Glyph {
    pub character: char,
    pub segments: [&'static str; 5],
}

#[derive(Debug, Clone)]
pub struct Font {
    pub glyphs: HashSet<Glyph>,
}

impl Font {
    pub fn new() -> Self {
        let glyphs = vec![
            Glyph {
                character: '0',
                segments: ["██████", "██  ██", "██  ██", "██  ██", "██████"],
            },
            Glyph {
                character: '1',
                segments: ["    ██", "    ██", "    ██", "    ██", "    ██"],
            },
            Glyph {
                character: '2',
                segments: ["██████", "    ██", "██████", "██    ", "██████"],
            },
            Glyph {
                character: '3',
                segments: ["██████", "    ██", "██████", "    ██", "██████"],
            },
            Glyph {
                character: '4',
                segments: ["██  ██", "██  ██", "██████", "    ██", "    ██"],
            },
            Glyph {
                character: '5',
                segments: ["██████", "██    ", "██████", "    ██", "██████"],
            },
            Glyph {
                character: '6',
                segments: ["██████", "██    ", "██████", "██  ██", "██████"],
            },
            Glyph {
                character: '7',
                segments: ["██████", "    ██", "    ██", "    ██", "    ██"],
            },
            Glyph {
                character: '8',
                segments: ["██████", "██  ██", "██████", "██  ██", "██████"],
            },
            Glyph {
                character: '9',
                segments: ["██████", "██  ██", "██████", "    ██", "██████"],
            },
            Glyph {
                character: ':',
                segments: ["  ", "██", "  ", "██", "  "],
            },
            Glyph {
                character: '.',
                segments: ["  ", "  ", "  ", "  ", "██"],
            },
        ]
        .into_iter()
        .collect();
        Font { glyphs }
    }
    fn render_string(&self, input: &str) -> Result<RenderedText, RenderedTextError> {
        let mut result = vec!["".to_string(); 5];
        for c in input.chars() {
            let glyph = self.glyphs.iter().find(|g| g.character == c);
            if let Some(glyph) = glyph {
                for i in 0..5 {
                    result[i].push_str(glyph.segments[i]);
                    result[i].push(' ');
                }
            } else {
                return Err(RenderedTextError::InvalidCharacter(c));
            }
        }
        Ok(RenderedText { lines: result })
    }
}

impl TryFrom<&str> for RenderedText {
    type Error = RenderedTextError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let font = Font::new();
        font.render_string(value)
    }
}

pub fn convert_style_vec(strings: Vec<String>, bold: bool) -> Vec<Vec<ratatui::style::Style>> {
    let space = Style::default();
    let stroke = if bold {
        Style::default().bg(Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Green)
    };
    let mut result = vec![];
    for s in strings {
        let mut v = vec![];
        for c in s.chars() {
            if c == ' ' {
                v.push(space);
            } else {
                v.push(stroke);
            }
        }
        result.push(v);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_style_vec() {
        let strings = vec![
            "██████ ██  ██ ██  ██ ██████ ".to_string(),
            "    ██ ██  ██ ██  ██     ██ ".to_string(),
            "██████ ██  ██ ██████ ██████ ".to_string(),
            "██████ ██  ██ ██████     ██ ".to_string(),
            "██  ██ ██  ██ ██████     ██ ".to_string(),
        ];
        let result = convert_style_vec(strings, false);
        for v in result {
            for s in v {
                print!("{:?}", s);
            }
            println!();
        }
    }

    #[test]
    fn test_digital_segments_for_time_string() {
        let font = Font::new();
        let segments = font.render_string("12:34").unwrap().lines;

        assert_eq!(
            segments,
            vec![
                "    ██ ██████    ██████ ██  ██ ".to_string(),
                "    ██     ██ ██     ██ ██  ██ ".to_string(),
                "    ██ ██████    ██████ ██████ ".to_string(),
                "    ██ ██     ██     ██     ██ ".to_string(),
                "    ██ ██████    ██████     ██ ".to_string(),
            ]
        );
    }

    #[test]
    fn test_digital_segments_for_colon_only() {
        let font = Font::new();
        let segments = font.render_string(":").unwrap().lines;

        assert_eq!(
            segments,
            vec![
                "   ".to_string(),
                "██ ".to_string(),
                "   ".to_string(),
                "██ ".to_string(),
                "   ".to_string(),
            ]
        );
    }

    #[test]
    fn test_render_string_rejects_invalid_characters() {
        let font = Font::new();
        let err = font.render_string("12a34").unwrap_err();
        assert_eq!(err, RenderedTextError::InvalidCharacter('a'));
    }
}
