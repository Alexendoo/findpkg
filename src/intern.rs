use bstr::{BStr, BString, ByteSlice, ByteVec};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Interner {
    offsets: HashMap<BString, u32>,
    buf: BString,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, s: &[u8]) -> u32 {
        let s = s.as_bstr();
        match self.offsets.get(s) {
            Some(&offset) => offset,
            None => {
                let offset = self.buf.len() as u32;

                self.buf.push_str(s);
                self.buf.push(b'\n');
                self.offsets.insert(s.to_owned(), offset);

                offset
            }
        }
    }

    pub fn get(&self, offset: u32) -> &BStr {
        let s = &self.buf[offset as usize..];
        let end = s.find_byte(b'\n').expect("Unterminated string");

        s[..end].as_bstr()
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

        let mut add_assert = |s: &str, offset| {
            let s = s.as_bytes();
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
