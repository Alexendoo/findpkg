use crate::Span;
use std::collections::HashMap;
use std::str;

#[derive(Debug, Default)]
pub struct Interner {
    spans: HashMap<String, Span>,
    buf: String,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, s: &str) -> Span {
        match self.spans.get(s) {
            Some(&span) => span,
            None => {
                let start = self.buf.len() as u32;
                let end = start + s.len() as u32;

                let span = Span { start, end };

                self.buf.push_str(s);
                self.spans.insert(s.to_string(), span);

                span
            }
        }
    }

    pub fn get(&self, span: Span) -> &str {
        span.get_str(self.buf())
    }

    pub fn buf(&self) -> &[u8] {
        self.buf.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn add() {
        let mut strings = Interner::new();

        let mut add_assert = |s, start, end| {
            let interned = strings.add(s);

            assert_eq!(strings.get(interned), s);
            assert_eq!(interned, Span { start, end });
        };

        add_assert("foo", 0, 3);
        add_assert("foo", 0, 3);

        add_assert("bar", 3, 6);
        add_assert("bar", 3, 6);

        add_assert("foo", 0, 3);

        assert_eq!(strings.buf(), b"foobar");
    }
}
