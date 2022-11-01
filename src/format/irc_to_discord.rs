use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
enum Token<'a> {
    Text(&'a str),
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Monospace,
    Color(Option<u8>, Option<u8>),
    ReverseColor,
    Reset,
}

impl<'a> Token<'a> {
    fn read_one(message: &'a str) -> Option<(Self, &'a str)> {
        if message.is_empty() {
            return None;
        }
        let (byte, split_message) = if message.is_char_boundary(1) {
            let (byte, message) = message.split_at(1);
            (Some(byte.as_bytes()[0]), message)
        } else {
            (None, message)
        };
        Some(
            byte.and_then(|byte| {
                let message = split_message;
                let token = match byte {
                    b'_' => Token::Text(r"\_"),
                    b'*' => Token::Text(r"\*"),
                    b'`' => Token::Text(r"\`"),
                    b'>' => Token::Text(r"\>"),
                    b'\\' => Token::Text(r"\\"),
                    0x02 => Token::Bold,
                    0x1d => Token::Italic,
                    0x1f => Token::Underline,
                    0x1e => Token::Strikethrough,
                    0x11 => Token::Monospace,
                    0x16 => Token::ReverseColor,
                    0x0f => Token::Reset,
                    0x03 => {
                        let msg_bytes = message.as_bytes();
                        let (fg, off, cont) =
                            match (msg_bytes.first(), msg_bytes.get(1), msg_bytes.get(2)) {
                                (Some(a @ b'0'..=b'9'), Some(b @ b'0'..=b'9'), Some(b',')) => {
                                    let a = a - b'0';
                                    let b = b - b'0';
                                    (Some(a * 10 + b), 2, true)
                                }
                                (Some(a @ b'0'..=b'9'), Some(b','), _) => (Some(a - b'0'), 1, true),
                                (Some(a @ b'0'..=b'9'), Some(b @ b'0'..=b'9'), _) => {
                                    let a = a - b'0';
                                    let b = b - b'0';
                                    (Some(a * 10 + b), 2, false)
                                }
                                (Some(a @ b'0'..=b'9'), _, _) => (Some(a - b'0'), 1, false),
                                _ => (None, 0, false),
                            };
                        if !cont {
                            return Some((Token::Color(fg, None), &message[off..]));
                        }

                        let msg_bytes = &msg_bytes[off..];
                        let (bg, off2) = match (msg_bytes.get(1), msg_bytes.get(2)) {
                            (Some(a @ b'0'..=b'9'), Some(b @ b'0'..=b'9')) => {
                                let a = a - b'0';
                                let b = b - b'0';
                                (Some(a * 10 + b), 3)
                            }
                            (Some(a @ b'0'..=b'9'), _) => (Some(a - b'0'), 2),
                            _ => (None, 0),
                        };
                        return Some((Token::Color(fg, bg), &message[(off + off2)..]));
                    }
                    _ => return None,
                };
                Some((token, message))
            })
            .unwrap_or_else(|| {
                let idx = message.find(|c| {
                    matches!(
                        c,
                        '_' | '*'
                            | '`'
                            | '>'
                            | '\\'
                            | '\x02'
                            | '\x1d'
                            | '\x1f'
                            | '\x1e'
                            | '\x11'
                            | '\x16'
                            | '\x0f'
                            | '\x03'
                    )
                });
                if let Some(idx) = idx {
                    let (text, next) = message.split_at(idx);
                    (Token::Text(text), next)
                } else {
                    (Token::Text(message), "")
                }
            }),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Formatting {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Monospace,
    Spoiler,
}

impl Formatting {
    fn as_str(&self) -> &'static str {
        use Formatting::*;

        match self {
            Bold => "**",
            Italic => "*",
            Underline => "__",
            Strikethrough => "~~",
            Monospace => "`",
            Spoiler => "||",
        }
    }
}

static SIMPLE_HTTP_REGEX: Lazy<Regex> = Lazy::new(|| {
    // http://www.regexguru.com/2008/11/detecting-urls-in-a-block-of-text/
    Regex::new(r"(?i:https?://[-a-z0-9+&@#/%?=~_|!:,.;]*[a-z0-9+&@#/%=~_|])").unwrap()
});

#[derive(Default)]
pub struct Converter {
    stack: Vec<Formatting>,
    message: String,
    fg: Option<u8>,
    bg: Option<u8>,
}

impl Converter {
    pub fn convert(message: impl AsRef<str>) -> String {
        let mut converter = Self::default();
        let message = message.as_ref();

        let matches_range = SIMPLE_HTTP_REGEX.find_iter(message).map(|m| m.range());
        let mut last_match_idx = 0usize;
        for url_range in matches_range {
            converter.push(&message[last_match_idx..url_range.start]);
            last_match_idx = url_range.end;

            // check this is really a URL
            let url_candidate = &message[url_range];
            if url::Url::parse(url_candidate).is_ok() {
                // push URL as-is
                converter.process_token(Token::Text(url_candidate));
            } else {
                converter.push(url_candidate);
            }
        }
        converter.push(&message[last_match_idx..]);
        converter.process_token(Token::Reset);

        converter.message
    }

    fn push(&mut self, mut message: &str) {
        while let Some((token, next)) = Token::read_one(message) {
            self.process_token(token);
            message = next;
        }
    }

    fn process_token(&mut self, token: Token) {
        match token {
            Token::Text(s) => self.message.push_str(s),
            Token::Bold => self.toggle_format(Formatting::Bold),
            Token::Italic => self.toggle_format(Formatting::Italic),
            Token::Underline => self.toggle_format(Formatting::Underline),
            Token::Strikethrough => self.toggle_format(Formatting::Strikethrough),
            Token::Monospace => self.toggle_format(Formatting::Monospace),
            Token::Color(fg, bg) => self.update_color(fg, bg),
            Token::ReverseColor => std::mem::swap(&mut self.fg, &mut self.bg),
            Token::Reset => {
                let stack = std::mem::take(&mut self.stack);
                stack
                    .into_iter()
                    .rev()
                    .for_each(|f| self.message.push_str(f.as_str()));
                self.fg = None;
                self.bg = None;
            }
        }
    }

    fn update_color(&mut self, new_fg: Option<u8>, new_bg: Option<u8>) {
        let spoilers_before = self.is_spoilers_state();
        self.fg = new_fg;
        if new_fg.is_none() || new_bg.is_some() {
            self.bg = new_bg;
        }
        if spoilers_before != self.is_spoilers_state() {
            self.toggle_format(Formatting::Spoiler);
        }
    }

    fn is_spoilers_state(&self) -> bool {
        self.fg.is_some() && self.fg == self.bg
    }

    fn toggle_format(&mut self, format: Formatting) {
        let Self { stack, message, .. } = self;
        if let Some(idx) = stack.iter().position(|&f| f == format) {
            stack[idx..]
                .iter()
                .rev()
                .chain(&stack[(idx + 1)..])
                .for_each(|f| message.push_str(f.as_str()));
            stack.remove(idx);
        } else if format != Formatting::Monospace && stack.last() == Some(&Formatting::Monospace) {
            message.push_str(Formatting::Monospace.as_str());
            stack.pop();
            message.push_str(format.as_str());
            stack.push(format);
            message.push_str(Formatting::Monospace.as_str());
            stack.push(Formatting::Monospace);
        } else {
            message.push_str(format.as_str());
            stack.push(format);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Converter;

    #[test]
    fn url_with_underline() {
        assert_eq!(
            Converter::convert("see this https://example.com/some_path_with_underline?and_this="),
            "see this https://example.com/some_path_with_underline?and_this=",
        );
        assert_eq!(
            Converter::convert("example.com/this_is_not_url wow really?"),
            "example.com/this\\_is\\_not\\_url wow really?",
        );
        assert_eq!(
            Converter::convert("http://example.com/multiple_urls https://example.com/should_work example.com/but_not_this"),
            "http://example.com/multiple_urls https://example.com/should_work example.com/but\\_not\\_this",
        );
    }
}
