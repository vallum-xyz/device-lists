# device-lists

Lists of USB maker codes (vendor IDs) and product codes (product IDs), with the names that go with them.

| File | What it holds | Licence |
|---|---|---|
| `keyboards.txt` | Keyboards, derived from the keyboard definitions of the VIA project | GPL-3.0 (`LICENSE`) |
| `devices.txt` | Makers and products of other device types that the public USB ID list does not have | CC0 1.0 (`LICENSE-CC0.txt`) |
| `devices-sources.tsv` | For every entry of `devices.txt`: where it is from and how sure it is | CC0 1.0 |
| `tools/` (except `tools/devices/`) | The converter that builds `keyboards.txt` | GPL-3.0 |
| `tools/devices/` | The script and the selection file that build `devices.txt` | CC0 1.0 |

The two lists are independent. Either can be used without the other.

## keyboards.txt

### Source and credit

All names and codes come from the VIA project's keyboard definitions:

- Repository: https://github.com/the-via/keyboards
- Commit: `95cba0b71deec91e4425046e28dd7dd573671d81`

Thanks to the VIA maintainers and to everyone who contributed a keyboard definition there.

### Licence

GNU General Public License, version 3, the same licence as the upstream repository. The full text is in `LICENSE`. This covers `keyboards.txt` and the converter in `tools/` (not `tools/devices/`).

### What is in the file

| | |
|---|---|
| Maker codes | 551 |
| Maker codes with a maker name | 102 |
| Products | 2,055 |
| Size | 42,930 bytes |
| SHA-256 | `dda7ff5630542d150e7064bb1477fa02679fbe0e207d0562e887b73e5e309d94` |

## devices.txt

Maker codes and product codes for mice, headsets and other audio gear, cameras, drives, security keys, hubs and docks, phones, game controllers, development boards and the chip makers whose codes turn up in unbranded devices. Only codes that the public USB ID list (`usb.ids`, version 2026.06.26) does not name are included.

### Source and credit

Every entry is a fact taken one at a time from a public source. None of the sources is copied whole. `SOURCES.md` describes each source and how entries were chosen; `devices-sources.tsv` gives the source of every single entry.

- Maker names: the USB Implementers Forum's list of assigned vendor IDs.
- Product names: the Linux kernel's `hid-ids.h`, Yubico's libfido2 `fidodevs`, and the SDL_GameControllerDB list.

Thanks to the people who maintain those.

### How sure an entry is

| Level | Meaning | Entries |
|---|---|---|
| registry | The maker name is the one the USB-IF lists for that vendor ID | 154 maker names |
| two sources | Two independent device-level sources give the same code pair | 2 products |
| one source | One device-level source gives the code pair | 599 products |

Nothing is included without a source.

### Licence

CC0 1.0 Universal: to the extent that the selection and arrangement in `devices.txt`, `devices-sources.tsv` and `tools/devices/` can be protected at all, those rights are waived. The full text is in `LICENSE-CC0.txt`. The names in the file are the makers' own names and product names; trademarks stay with their owners.

### What is in the file

| | |
|---|---|
| Maker codes | 241 |
| Maker codes with a maker name | 154 |
| Products | 601 |
| Size | 21,160 bytes |
| SHA-256 | `35e9bbd465cfc22dfcbf5352c617553475800803bc947c6cc689ea58b8fb148d` |

Maker names by the kind of device the maker is mostly known for: audio 21, chip makers 16, game controllers 14, storage 13, mice 12, other input devices 12, cameras 11, computers 11, hubs and docks 10, keyboards 7, security keys 6, development boards 5, phones 5, network 3, other 8.

Products by source: 293 game controllers (SDL_GameControllerDB), 246 input devices (Linux kernel), 64 security keys (libfido2). Two products are in two sources.

## Format

Both lists use the same format. Plain text, UTF-8, lines ending in a line feed, fields separated by one tab.

```
# vallum-pack keyboards 1
vvvv<tab>maker name
<tab>pppp<tab>product name
```

- The first line is always exactly `# vallum-pack keyboards 1` in `keyboards.txt` and `# vallum-pack devices 1` in `devices.txt`.
- `vvvv` is the USB vendor ID and `pppp` the USB product ID, each as four lower-case hex digits.
- A maker line is followed by the product lines of that maker code. Maker codes and the products under each are in ascending order.
- The maker name is `-` where the file names no maker for the code.
  - In `keyboards.txt` that is the case for codes shared by many makers (for example `1209` and `feed`), for codes that the public USB ID list already names, and for codes where the definitions do not point to one maker. Where a maker name is given there, it was derived from the upstream folder name and the product names, not taken from a registry.
  - In `devices.txt` that is the case for codes that the public USB ID list already names; the line is only there to carry products. Where a maker name is given, it is the registered one.

`devices-sources.tsv` has a heading line and four tab-separated columns: the code (`vvvv` for a maker, `vvvv:pppp` for a product), the name, how sure, and the address of the source or sources.

## Accuracy

A vendor ID says who the code is registered to, which is not always the brand printed on the device: many products carry the code of the company that made the chip or built the device for the brand. A code can be used by more than one product, and anyone can make a device that reports any code. A match says what a code is registered to or commonly used for, nothing more.

Product names are taken from the sources as they are and may contain errors. Derived maker names in `keyboards.txt` can be misspelled.

To report a wrong entry, open an issue in this repository and give the code, what the list says, what it should say, and a source. If the error is in an upstream source, please report it there as well.

## Rebuilding keyboards.txt

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

## Rebuilding devices.txt

`tools/devices/make_devices.py` (Python 3, no packages) builds the file from `tools/devices/select.txt` and the downloaded sources. `SOURCES.md` lists the inputs and the steps. The sources change over time, so a later download can give a different file.
