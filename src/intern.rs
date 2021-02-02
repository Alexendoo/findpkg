use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: u32,
    end: u32,
}

#[derive(Debug, Default)]
pub struct Interner {
    spans: HashMap<String, Span>,
    buf: String,
}

impl Interner {
    pub fn new() -> Self {
        Default::default()
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

    pub fn buf(&self) -> &str {
        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut strings = Interner::new();

        assert_eq!(strings.add("foo"), Span { start: 0, end: 3 });
        assert_eq!(strings.add("bar"), Span { start: 3, end: 6 });

        assert_eq!(strings.buf(), "foobar");
    }
}
