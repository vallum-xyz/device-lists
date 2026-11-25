// SPDX-License-Identifier: GPL-3.0-or-later
// Part of the converter that rebuilds keyboards.txt from https://github.com/the-via/keyboards.

//! Reads one VIA keyboard definition without a JSON library: only the three fields the pack needs.
//! The files are someone else's data. Nothing here can panic on odd input; a file that does not fit gives None.

use crate::names::{clean_name, parse_id};

struct Scan<'a> {
    b: &'a [u8],
    i: usize,
}

fn space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r')
}

impl Scan<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && space(self.b[self.i]) {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// A JSON string with its escapes resolved. None when it is not well formed.
    fn string(&mut self) -> Option<String> {
        self.ws();
        if self.b.get(self.i) != Some(&b'"') {
            return None;
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.b.get(self.i)?;
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).ok(),
                b'\\' => {
                    let e = *self.b.get(self.i)?;
                    self.i += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'n' | b'r' | b't' | b'b' | b'f' => ' ',
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i.checked_add(4)?)?).ok()?;
                            self.i += 4;
                            // a lone half of a pair has no character of its own: shown as a space
                            char::from_u32(u32::from_str_radix(hex, 16).ok()?).unwrap_or(' ')
                        }
                        _ => return None,
                    };
                    out.extend(ch.encode_utf8(&mut [0u8; 4]).as_bytes());
                }
                _ => out.push(c),
            }
        }
    }

    /// Steps over any value: a string, a number or word, or a whole object or array.
    fn skip(&mut self) -> Option<()> {
        self.ws();
        match *self.b.get(self.i)? {
            b'"' => self.string().map(|_| ()),
            b'{' | b'[' => {
                let mut depth = 0usize;
                loop {
                    match *self.b.get(self.i)? {
                        b'"' => {
                            self.string()?;
                            continue;
                        }
                        b'{' | b'[' => depth += 1,
                        b'}' | b']' => {
                            depth = depth.checked_sub(1)?;
                            if depth == 0 {
                                self.i += 1;
                                return Some(());
                            }
                        }
                        _ => {}
                    }
                    self.i += 1;
                }
            }
            _ => {
                let from = self.i;
                while self.i < self.b.len() && !space(self.b[self.i]) && !matches!(self.b[self.i], b',' | b'}' | b']') {
                    self.i += 1;
                }
                if self.i > from { Some(()) } else { None }
            }
        }
    }
}

/// The maker code, product code and name from one definition: the `vendorId`, `productId` and `name` of the
/// outermost object, where the codes are text such as "0x3434". Everything else in the file is stepped over.
/// The first of a repeated key stands.
pub fn via_fields(json: &str) -> Option<(u16, u16, String)> {
    let mut s = Scan { b: json.trim_start_matches('\u{feff}').as_bytes(), i: 0 };
    if !s.eat(b'{') {
        return None;
    }
    let (mut vid, mut pid, mut name) = (None, None, None);
    if !s.eat(b'}') {
        loop {
            let key = s.string()?;
            if !s.eat(b':') {
                return None;
            }
            s.ws();
            let wanted = matches!(key.as_str(), "vendorId" | "productId" | "name");
            if wanted && s.b.get(s.i) == Some(&b'"') {
                let v = s.string()?;
                match key.as_str() {
                    "vendorId" => vid = vid.or(parse_id(&v)),
                    "productId" => pid = pid.or(parse_id(&v)),
                    _ => name = name.or(clean_name(&v)),
                }
            } else {
                s.skip()?;
            }
            if s.eat(b',') {
                continue;
            }
            if s.eat(b'}') {
                break;
            }
            return None;
        }
    }
    Some((vid?, pid?, name?))
}
