use std::collections::HashMap;
use std::str;

#[derive(Debug, Default)]
pub struct Interner {
    offsets: HashMap<String, u32>,
    buf: String,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, s: &str) -> u32 {
        match self.offsets.get(s) {
            Some(&offset) => offset,
            None => {
                let offset = self.buf.len() as u32;

                self.buf.push_str(s);
                self.buf.push('\n');
                self.offsets.insert(s.to_string(), offset);

                offset
            }
        }
    }

    pub fn get(&self, offset: u32) -> &str {
        let s = &self.buf[offset as usize..];
        let end = s.find('\n').expect("Unterminated string");

        &s[..end]
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

        let mut add_assert = |s, offset| {
            let interned = strings.add(s);

            assert_eq!(strings.get(interned), s);
            assert_eq!(interned, offset);
        };

        add_assert("foo", 0);
        add_assert("foo", 0);

        add_assert("bar", 4);
        add_assert("bar", 4);

        add_assert("foo", 0);

        assert_eq!(strings.buf(), b"foo\nbar\n");
    }
}
