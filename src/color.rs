//! ANSI 终端颜色

/// ANSI 终端颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Gray,
    Magenta,
    Bold,
    Reset,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "\x1b[31m"),
            Color::Green => write!(f, "\x1b[32m"),
            Color::Yellow => write!(f, "\x1b[33m"),
            Color::Blue => write!(f, "\x1b[34m"),
            Color::Cyan => write!(f, "\x1b[36m"),
            Color::Gray => write!(f, "\x1b[38;5;8m"),
            Color::Magenta => write!(f, "\x1b[35m"),
            Color::Bold => write!(f, "\x1b[1m"),
            Color::Reset => write!(f, "\x1b[0m"),
        }
    }
}

/// 用 ANSI 颜色包裹文本
pub fn styled(text: &str, color: Color) -> String {
    format!("{color}{text}{reset}", color = color, reset = Color::Reset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_ansi() {
        assert_eq!(
            format!("{}hello{}", Color::Red, Color::Reset),
            "\x1b[31mhello\x1b[0m"
        );
    }

    #[test]
    fn styled_wraps_text() {
        assert_eq!(styled("hi", Color::Green), "\x1b[32mhi\x1b[0m");
    }
}
