# device-lists

A list of USB maker codes and product codes for keyboards, with the names that go with them.

`keyboards.txt` is derived from the keyboard definitions of the VIA project.

## Source and credit

All names and codes come from the VIA project's keyboard definitions:

- Repository: https://github.com/the-via/keyboards
- Commit: `95cba0b71deec91e4425046e28dd7dd573671d81`

Thanks to the VIA maintainers and to everyone who contributed a keyboard definition there.

## Licence

GNU General Public License, version 3, the same licence as the upstream repository. The full text is in `LICENSE`. This covers `keyboards.txt` and the converter in `tools/`.

## What is in the file

| | |
|---|---|
| Maker codes | 551 |
| Maker codes with a maker name | 102 |
| Products | 2,055 |
| Size | 42,930 bytes |
| SHA-256 | `dda7ff5630542d150e7064bb1477fa02679fbe0e207d0562e887b73e5e309d94` |

## Format

Plain text, UTF-8, lines ending in a line feed, fields separated by one tab.

```
# vallum-pack keyboards 1
vvvv<tab>maker name
<tab>pppp<tab>product name
```

- The first line is always exactly `# vallum-pack keyboards 1`.
- `vvvv` is the USB vendor ID and `pppp` the USB product ID, each as four lower-case hex digits.
- A maker line is followed by the product lines of that maker code. Maker codes and the products under each are in ascending order.
- The maker name is `-` where no single maker can be named for the code. That is the case for codes shared by many makers (for example `1209` and `feed`), for codes that the public USB ID list already names, and for codes where the definitions do not point to one maker.
- Where a maker name is given, it was derived from the upstream folder name and the product names, not taken from a registry.

## Accuracy

Product names are taken from the upstream definitions as they are and may contain errors. Derived maker names can be misspelled. A code can be used by more than one product, so a match says what a code is commonly used for, nothing more.

To report a wrong entry, open an issue in this repository. If the error is in the upstream definition, please report it to the VIA project as well.

## Rebuilding the file

`tools/` holds the converter, a small Rust program with no dependencies. It needs two inputs that are not part of this repository:

1. An unpacked copy of https://github.com/the-via/keyboards at the commit named above (the folder that holds `v3` and `src`).
2. The public USB ID list `usb.ids`, version 2026.06.26, from http://www.linux-usb.org/usb.ids. It is used only to decide which maker codes already have a name there; for those codes no maker name is derived.

```
cd tools
cargo build --release
target/release/device-lists-convert <keyboards folder> <usb.ids file> keyboards.txt
```

With those two inputs the output is byte for byte the `keyboards.txt` in this repository; compare the SHA-256 above. A different upstream commit or a different `usb.ids` version can give a different file.

The converter reads the definition files as data only. It takes the `vendorId`, `productId` and `name` of each definition, reads `v3` before `src`, and keeps the first definition of a repeated code. Files that do not give a name and two codes are skipped (13 of 3,515 at this commit).
