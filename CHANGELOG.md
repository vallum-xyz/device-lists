# Changelog

## 2026-11-25 (second release)

- Added `devices.txt`: 241 maker codes (154 with the maker name the USB-IF lists) and 601 products that the public USB ID list does not have. Sources per entry in `devices-sources.tsv`, described in `SOURCES.md`. Licence CC0 1.0 (`LICENSE-CC0.txt`).
- Added the build script and selection file for it in `tools/devices/`.
- `keyboards.txt` is unchanged.

## 2026-11-25

- First version of `keyboards.txt`: 551 maker codes and 2,055 products, from https://github.com/the-via/keyboards at commit `95cba0b71deec91e4425046e28dd7dd573671d81`.
- Converter added in `tools/`.
