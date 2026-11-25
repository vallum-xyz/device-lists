# Sources of devices.txt

All sources were downloaded on 2026-11-25. Each entry of `devices.txt` is listed in `devices-sources.tsv` with its source and how sure it is.

Only codes and names are taken, one entry at a time, and only for codes the public USB ID list (`usb.ids`, version 2026.06.26, http://www.linux-usb.org/usb.ids) does not already name. No source is copied whole, and no comments, code or other text from a source is included.

## Maker names: USB-IF list of assigned vendor IDs ("registry")

- Publisher: USB Implementers Forum, the body that assigns USB vendor IDs.
- Address: https://www.usb.org/sites/default/files/vendor_ids10282026.pdf (linked from https://www.usb.org/developers).
- What is taken: the company name for 154 vendor IDs, out of roughly 13,900 in the list.
- How they were chosen: 136 vendor IDs were picked by hand as makers of common devices or of chips found in unbranded devices (`tools/devices/select.txt`, which holds codes and a device type but no names). A further 18 are vendor IDs that the Linux kernel's `hid-ids.h` names and whose kernel name shares a distinctive word with the registered name. The name written is always the registered one.
- One name is corrected for a character the text extraction lost: `349e` is written "Token2 Sàrl".
- The registered name is the legal name of the company that holds the vendor ID. It can differ from the brand on the product.

## Product names ("one source", "two sources")

A product is "two sources" when two of the three sources below give the same vendor ID and product ID, otherwise "one source".

### Linux kernel, drivers/hid/hid-ids.h

- Address: https://github.com/torvalds/linux/blob/master/drivers/hid/hid-ids.h
- Licence of the source: GPL-2.0.
- What is taken: 246 pairs of vendor ID and product ID with a product label read from the constant's name (for example `USB_DEVICE_ID_NINTENDO_NS2_PROCON` gives "NS2 Procon"). A label is only kept when it has two or more words and at least one word of four letters or more; internal board code names are left out.
- These labels are the kernel developers' short names for the device, not the name printed on it.

### libfido2, udev/fidodevs

- Address: https://github.com/Yubico/libfido2/blob/main/udev/fidodevs
- Licence of the source: BSD-2-Clause, Copyright (c) Yubico AB.
- What is taken: 64 security keys (vendor ID, product ID, name). Entries the source itself calls unknown are left out.

### SDL_GameControllerDB, gamecontrollerdb.txt

- Address: https://github.com/mdqinc/SDL_GameControllerDB/blob/master/gamecontrollerdb.txt
- Licence of the source: zlib licence.
- What is taken: 293 game controllers. The vendor ID and product ID are read from the identifier at the start of each line, together with the name on that line. A name is only kept when it shares a distinctive word with the maker of that vendor ID and says more than the maker's name. Button mappings are not taken.

## Rebuilding

1. Download the four files above and `usb.ids`.
2. Turn the USB-IF PDF into text with `pdftotext -simple` (the `-layout` mode shifts some numbers by one row and must not be used).
3. Place them as `usbif/v_simple.txt`, `kernel/hid-ids.h`, `fido/fidodevs` and `sdl/gamecontrollerdb.txt` in one folder.
4. Convert `usb.ids` to lines of `vvvv<tab>maker name` and `<tab>pppp<tab>product name` (codes and names only).
5. Run `python -I tools/devices/make_devices.py <converted usb.ids> keyboards.txt <sources folder> <output folder>`.

The script prints the counts and writes `devices.txt` and `devices-sources.tsv`.
