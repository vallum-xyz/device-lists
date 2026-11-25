// SPDX-License-Identifier: GPL-3.0-or-later
// Part of the converter that rebuilds keyboards.txt from https://github.com/the-via/keyboards.

//! Names and codes, the maker codes of the public USB list, and the grouping of keyboard definitions by maker code.

use std::collections::{BTreeMap, HashMap, HashSet};

const MAX_NAME: usize = 120;

/// Keeps a name printable and on one line. Returns None when nothing is left.
pub fn clean_name(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut gap = false;
    for c in s.chars() {
        if c.is_control() || c.is_whitespace() {
            gap = !out.is_empty();
            continue;
        }
        if gap {
            out.push(' ');
            gap = false;
        }
        out.push(c);
        if out.chars().count() >= MAX_NAME {
            break;
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// "0x3434" as a number. Anything else is refused.
pub fn parse_id(s: &str) -> Option<u16> {
    let h = s.trim().strip_prefix("0x").or_else(|| s.trim().strip_prefix("0X"))?;
    if h.is_empty() || h.len() > 4 || !h.bytes().all(|b| b.is_ascii_hexdigit()) { None } else { u16::from_str_radix(h, 16).ok() }
}

fn hex4(s: &str) -> Option<u16> {
    if s.len() == 4 && s.bytes().all(|b| b.is_ascii_hexdigit()) { u16::from_str_radix(s, 16).ok() } else { None }
}

/// The maker codes in `usb.ids` ("vvvv  maker" lines). Reading stops at the class list that follows the makers
/// (first line starting with "C ").
pub fn usb_ids_makers(text: &str) -> HashSet<u16> {
    let mut out = HashSet::new();
    for line in text.lines() {
        if line.starts_with("C ") {
            break;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with('\t') {
            continue;
        }
        let (Some(code), Some(name)) = (line.get(..4), line.get(4..)) else { continue };
        if !name.starts_with(' ') {
            continue;
        }
        if let (Some(vid), Some(_)) = (hex4(code), clean_name(name)) {
            out.insert(vid);
        }
    }
    out
}

/// One keyboard definition: its codes, its name and the maker folder it sits in.
#[derive(Debug, Clone, PartialEq)]
pub struct KbEntry {
    pub vid: u16,
    pub pid: u16,
    pub name: String,
    pub folder: String,
}

/// One maker code. `maker` is None when the code is shared or no one maker stands out.
#[derive(Debug, Clone, PartialEq)]
pub struct KbVendor {
    pub vid: u16,
    pub maker: Option<String>,
    pub products: Vec<(u16, String)>,
}

/// Maker codes that many unrelated makers use (community and test codes). Product names are kept, no maker is derived.
pub const SHARED_VIDS: &[u16] = &[0x1209, 0xfeed, 0x16c0, 0x1d50, 0x0000, 0xffff, 0xf055, 0x6666, 0x1234, 0xabcd];

fn letters(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// Groups the definitions by maker code. A maker name is given only where one maker clearly holds the code:
/// the code is not a shared one, is not already in the public USB list (`in_usb`), has at least 3 products,
/// and at least 9 in 10 of them sit in one maker folder. The first definition of a repeated code stands.
pub fn derive_keyboards(entries: &[KbEntry], in_usb: &dyn Fn(u16) -> bool) -> Vec<KbVendor> {
    let mut by: BTreeMap<u16, Vec<&KbEntry>> = BTreeMap::new();
    for e in entries {
        let list = by.entry(e.vid).or_default();
        if clean_name(&e.name).is_some() && !list.iter().any(|x| x.pid == e.pid) {
            list.push(e);
        }
    }
    let mut out = Vec::new();
    for (vid, list) in by {
        if list.is_empty() {
            continue;
        }
        let mut maker = None;
        if !SHARED_VIDS.contains(&vid) && !in_usb(vid) && list.len() >= 3 {
            let mut folders: HashMap<&str, usize> = HashMap::new();
            for e in &list {
                *folders.entry(e.folder.as_str()).or_default() += 1;
            }
            let (folder, n) = folders.into_iter().max_by_key(|(f, n)| (*n, std::cmp::Reverse(*f))).unwrap();
            if n * 10 >= list.len() * 9 && !matches!(letters(folder).as_str(), "" | "other" | "misc" | "unknown" | "handwired" | "example" | "test") {
                // the spelling the product names use, when most of them start with the name of the folder
                let mut first: HashMap<&str, usize> = HashMap::new();
                for e in list.iter().filter(|e| e.folder == folder) {
                    if let Some(w) = e.name.split_whitespace().next() {
                        *first.entry(w).or_default() += 1;
                    }
                }
                let spelled = first.into_iter().filter(|(w, _)| letters(w) == letters(folder)).max_by_key(|(w, c)| (*c, std::cmp::Reverse(*w)));
                maker = match spelled {
                    Some((w, c)) if c * 10 >= n * 8 => clean_name(w),
                    _ => clean_name(&folder.replace('_', " ")),
                };
            }
        }
        let mut products: Vec<(u16, String)> = list.iter().filter_map(|e| Some((e.pid, clean_name(&e.name)?))).collect();
        products.sort_by_key(|p| p.0);
        out.push(KbVendor { vid, maker, products });
    }
    out
}

/// "vvvv<tab>maker" (or "vvvv<tab>-" when there is no maker) and "<tab>pppp<tab>product" lines.
pub fn write_keyboards(v: &[KbVendor]) -> String {
    let mut s = String::new();
    for m in v {
        s.push_str(&format!("{:04x}\t{}\n", m.vid, m.maker.as_deref().unwrap_or("-")));
        for (pid, name) in &m.products {
            s.push_str(&format!("\t{:04x}\t{}\n", pid, name));
        }
    }
    s
}
