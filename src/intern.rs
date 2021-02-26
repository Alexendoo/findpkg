use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn get(self, buf: &[u8]) -> &str {
        let slice = &buf[self.start as usize..self.end as usize];

        str::from_utf8(slice).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InternedStr {
    pub str: &'static str,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Interner {
    spans: HashMap<&'static str, Span>,
    buf: String,
}

impl Interner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, s: &str) -> InternedStr {
        match self.spans.get_key_value(s) {
            Some((&key, &span)) => InternedStr { str: key, span },
            None => {
                let start = self.buf.len() as u32;
                let end = start + s.len() as u32;

                let span = Span { start, end };

                let leaked = Box::leak(s.to_string().into_boxed_str());

                self.buf.push_str(leaked);
                self.spans.insert(leaked, span);

                InternedStr { str: leaked, span }
            }
        }
    }

    pub fn buf(&self) -> &[u8] {
        self.buf.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut strings = Interner::new();

        let mut add_assert = |s, start, end| {
            let interned = strings.add(s);

            assert_eq!(interned.str, s);
            assert_eq!(interned.span, Span { start, end });
        };

        add_assert("foo", 0, 3);
        add_assert("foo", 0, 3);

        add_assert("bar", 3, 6);
        add_assert("bar", 3, 6);

        add_assert("foo", 0, 3);

        assert_eq!(strings.buf(), "foobar");
    }
}
