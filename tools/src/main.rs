// SPDX-License-Identifier: GPL-3.0-or-later
// Rebuilds keyboards.txt from https://github.com/the-via/keyboards.

//! Usage: device-lists-convert <unpacked the-via/keyboards folder> <usb.ids file> <output file>
//!
//! Reads the keyboard definitions as data only; nothing in the folder is run.

mod names;
mod via;

use names::KbEntry;
use std::{env, fs, path::Path, process::exit};

const HEADER: &str = "# vallum-pack keyboards 1";
const MAX_DEFINITION_BYTES: u64 = 4 * 1024 * 1024;
const MAX_DEFINITION_FILES: usize = 50_000;
const MAX_FOLDER_DEPTH: usize = 8;

fn walk(dir: &Path, folder: Option<&str>, depth: usize, out: &mut Vec<KbEntry>, skipped: &mut usize, files: &mut usize) -> Result<(), String> {
    if depth > MAX_FOLDER_DEPTH {
        return Ok(());
    }
    let Ok(rd) = fs::read_dir(dir) else { return Ok(()) };
    let mut items: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    items.sort();
    for p in items {
        let Some(file) = p.file_name().and_then(|n| n.to_str()) else { continue };
        // links are not followed
        let Ok(meta) = fs::symlink_metadata(&p) else { continue };
        if meta.is_dir() {
            walk(&p, Some(folder.unwrap_or(file)), depth + 1, out, skipped, files)?;
        } else if meta.is_file() && file.ends_with(".json") {
            *files += 1;
            if *files > MAX_DEFINITION_FILES {
                return Err("the folder holds too many files".into());
            }
            let Some(folder) = folder else { continue };
            let text = if meta.len() <= MAX_DEFINITION_BYTES { fs::read_to_string(&p).ok() } else { None };
            match text.as_deref().and_then(via::via_fields) {
                Some((vid, pid, name)) => out.push(KbEntry { vid, pid, name, folder: folder.to_string() }),
                None => *skipped += 1,
            }
        }
    }
    Ok(())
}

fn run(root: &Path, usb_ids: &Path, out: &Path) -> Result<String, String> {
    let (v3, src) = (root.join("v3"), root.join("src"));
    if !v3.is_dir() && !src.is_dir() {
        return Err("the folder does not hold the keyboard definitions (no v3 or src folder)".into());
    }
    let usb = fs::read(usb_ids).map_err(|e| format!("cannot read the USB list: {e}"))?;
    let in_usb = names::usb_ids_makers(&String::from_utf8_lossy(&usb));
    if in_usb.len() < 2000 {
        return Err(format!("the USB list holds only {} maker codes; it does not look like usb.ids", in_usb.len()));
    }
    let (mut entries, mut skipped, mut files) = (Vec::new(), 0usize, 0usize);
    // the newer definitions first: the first definition of a repeated code stands
    walk(&v3, None, 0, &mut entries, &mut skipped, &mut files)?;
    walk(&src, None, 0, &mut entries, &mut skipped, &mut files)?;
    let makers = names::derive_keyboards(&entries, &|vid| in_usb.contains(&vid));
    let text = format!("{HEADER}\n{}", names::write_keyboards(&makers));
    fs::write(out, &text).map_err(|e| format!("cannot write the output file: {e}"))?;
    Ok(format!(
        "{} bytes; {} definitions read, {} skipped; {} maker codes ({} with a maker name), {} products",
        text.len(),
        entries.len(),
        skipped,
        makers.len(),
        makers.iter().filter(|m| m.maker.is_some()).count(),
        makers.iter().map(|m| m.products.len()).sum::<usize>()
    ))
}

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 4 {
        eprintln!("usage: device-lists-convert <unpacked the-via/keyboards folder> <usb.ids file> <output file>");
        exit(2);
    }
    match run(Path::new(&a[1]), Path::new(&a[2]), Path::new(&a[3])) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("nothing written: {e}");
            exit(1);
        }
    }
}
