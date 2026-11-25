#!/usr/bin/env python3
"""Builds devices.txt: USB maker codes and product codes that the public USB ID list does not have.

Run with -I (the inputs are files fetched from the internet and are read as text only):

  python -I make_devices.py <usb.txt> <keyboards.txt> <sources folder> <out folder>

<sources folder> holds, fetched by hand (see SOURCES.md for the addresses and dates):
  usbif/v_simple.txt        the USB-IF list of assigned vendor IDs, as text (pdftotext -simple of their PDF)
  kernel/hid-ids.h          drivers/hid/hid-ids.h of the Linux kernel
  fido/fidodevs             udev/fidodevs of Yubico's libfido2
  sdl/gamecontrollerdb.txt  the SDL_GameControllerDB list

Every entry carries where it is from and how sure it is, in devices-sources.tsv:
  registry      the maker's name is the one the USB-IF lists for that vendor ID
  two sources   two of the device-level sources give the same code pair
  one source    one of them does
Nothing is taken on a guess, and no source is copied whole: an entry is only written when its code is missing from
the public list and it passes the checks below.
"""
import io
import os
import re
import sys

usb_path, kb_path, src, out = sys.argv[1:5]

GENERIC = set("""co ltd inc corp corporation company limited gmbh ag sa bv ab oy as llc pte pty plc srl sarl spa kg
technology technologies tech electronics electronic electric international industrial industries industry group holding
holdings the and of shenzhen dongguan guangzhou hangzhou shanghai beijing wuxi zhuhai hong kong hk china taiwan asia
pacific europe usa systems system solutions products devices device labs lab information science digital trading
innovations innovation microelectronics semiconductor semiconductors communication communications mobile network
networks computer computers data storage memory gaming gear studio usb id vendor hid controller gamepad wireless
wired pro mini micro for with""".split())


def toks(s):
    return [t for t in re.findall(r"[a-z0-9]+", s.lower()) if len(t) >= 3 and t not in GENERIC and not t.isdigit()]


def shares(a, b):
    """True when two names clearly mean the same maker: a distinctive word of one is in the other."""
    ta, tb = toks(a), toks(b)
    ja, jb = "".join(re.findall(r"[a-z0-9]+", a.lower())), "".join(re.findall(r"[a-z0-9]+", b.lower()))
    return any(t in tb for t in ta) or any(len(t) >= 5 and t in jb for t in ta) or any(len(t) >= 5 and t in ja for t in tb)


def clean(name):
    name = re.sub(r"\s+", " ", name.replace("\t", " ")).strip()
    if not name or len(name) > 120 or "�" in name or any(ord(c) < 32 for c in name):
        return None
    return name


# ----- what the public list and the keyboard list already have -----
usb_makers, usb_products = {}, set()
cur = None
for line in io.open(usb_path, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    if line.startswith("\t"):
        p = line[1:].split("\t")
        if cur and len(p) >= 2:
            usb_products.add((cur, p[0]))
    else:
        p = line.split("\t")
        if len(p) >= 2 and re.fullmatch(r"[0-9a-f]{4}", p[0]):
            cur = p[0]
            usb_makers[cur] = p[1]
kb_products, cur = set(), None
for line in io.open(kb_path, encoding="utf-8"):
    line = line.rstrip("\n")
    if line.startswith("#") or not line:
        continue
    if line.startswith("\t"):
        kb_products.add((cur, line[1:].split("\t")[0]))
    else:
        cur = line.split("\t")[0]

# ----- the USB-IF list -----
FIX = {"349e": "Token2 Sàrl"}  # the one name whose accented letter the text extraction lost; the rest are dropped if damaged
reg = {}
for line in io.open(os.path.join(src, "usbif", "v_simple.txt"), encoding="utf-8", errors="replace"):
    m = re.match(r"^(\S.*?)\s{2,}(\d{1,5})\s*$", line.rstrip("\n").replace("\x0c", ""))
    if m and int(m.group(2)) <= 0xFFFF:
        reg.setdefault("%04x" % int(m.group(2)), []).append(m.group(1).strip())
for k, v in FIX.items():
    if k in reg:
        reg[k] = [v]
USBIF = "https://www.usb.org/sites/default/files/vendor_ids10282026.pdf"

makers = {}    # vid -> (name or None, type, confidence, source)
products = {}  # (vid, pid) -> (name, confidence, source)
dropped = []


def add_maker(vid, kind, why):
    if vid in usb_makers or vid in makers:
        return vid in makers
    names = reg.get(vid)
    if not names or len(names) != 1:
        dropped.append((vid, "not one name in the USB-IF list", why))
        return False
    name = clean(names[0])
    if not name:
        dropped.append((vid, "name damaged in extraction", why))
        return False
    makers[vid] = (name, kind, "registry", USBIF)
    return True


# 1. the chosen makers
here = os.path.dirname(os.path.abspath(__file__))
for line in io.open(os.path.join(here, "select.txt"), encoding="utf-8"):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    vid, kind = line.split("\t")
    add_maker(vid.lower(), kind, "chosen")

# 2. the Linux kernel's HID id header: makers whose kernel name agrees with the registered name, and their devices
KERNEL = "https://github.com/torvalds/linux/blob/master/drivers/hid/hid-ids.h"
kv, kd = {}, []
for line in io.open(os.path.join(src, "kernel", "hid-ids.h"), encoding="utf-8", errors="replace"):
    m = re.match(r"#define\s+USB_VENDOR_ID_(\w+)\s+0x([0-9a-fA-F]{4})\b", line)
    if m:
        kv[m.group(1)] = m.group(2).lower()
        continue
    m = re.match(r"#define\s+USB_DEVICE_ID_(\w+)\s+0x([0-9a-fA-F]{4})\b", line)
    if m:
        kd.append((m.group(1), m.group(2).lower()))
kernel_pairs = {}
for macro, pid in kd:
    best = max((v for v in kv if macro.startswith(v + "_")), key=len, default=None)
    if not best:
        continue
    vid = kv[best]
    words = macro[len(best) + 1:].split("_")
    label = " ".join(w if re.search(r"\d", w) or len(w) <= 3 else w.capitalize() for w in words)
    kernel_pairs.setdefault((vid, pid), (best, label))
for macro, vid in kv.items():
    if vid not in usb_makers and vid in reg and len(reg[vid]) == 1 and shares(macro.replace("_", " "), reg[vid][0]):
        add_maker(vid, "input", "kernel")


def known_maker(vid):
    return usb_makers.get(vid) or (makers.get(vid) or (None,))[0]


for (vid, pid), (vmacro, label) in kernel_pairs.items():
    if (vid, pid) in usb_products or (vid, pid) in kb_products or not known_maker(vid):
        continue
    if not shares(vmacro.replace("_", " "), known_maker(vid)):
        continue  # the kernel files this device under another name than the code's maker: left out
    name = clean(label)
    # the kernel's identifiers are working names: keep only the ones that read as a product name (a word of four
    # letters or more, not a bare number, not one of Apple's internal board names)
    if (name and len(name.split()) >= 2 and re.search(r"[A-Za-z]{4,}", name)
            and not re.search(r"WELLSPRING|GEYSER|FOUNTAIN|^ALU |(First|Last)", name, re.I)):
        products[(vid, pid)] = (name, "one source", KERNEL)

# 3. Yubico's list of FIDO security keys
FIDO = "https://github.com/Yubico/libfido2/blob/main/udev/fidodevs"
fv = {}
for line in io.open(os.path.join(src, "fido", "fidodevs"), encoding="utf-8", errors="replace"):
    m = re.match(r"vendor\s+(\w+)\s+0x([0-9a-fA-F]{4})\s+(.+)", line)
    if m:
        fv[m.group(1)] = (m.group(2).lower(), m.group(3).strip())
        continue
    m = re.match(r"product\s+(\w+)\s+0x([0-9a-fA-F]{4})\s+(.+)", line)
    if m and m.group(1) in fv:
        vid, vname = fv[m.group(1)]
        pid, name = m.group(2).lower(), clean(m.group(3))
        if name and name.lower().startswith("unknown"):
            name = None
        if vname.lower().startswith("unknown"):
            continue
        if vid not in usb_makers and vid not in makers:
            if vid in reg and len(reg[vid]) == 1 and shares(vname, reg[vid][0]):
                add_maker(vid, "key", "fido")
            elif clean(vname) and vid not in reg:
                makers[vid] = (clean(vname), "key", "one source", FIDO)
        if name and (vid, pid) not in usb_products and (vid, pid) not in kb_products and known_maker(vid):
            if (vid, pid) in products and products[(vid, pid)][2] != FIDO:
                products[(vid, pid)] = (name, "two sources", products[(vid, pid)][2] + " ; " + FIDO)
            else:
                products[(vid, pid)] = (name, "one source", FIDO)

# 4. the SDL controller list: only controllers whose name agrees with the maker of their code
SDL = "https://github.com/mdqinc/SDL_GameControllerDB/blob/master/gamecontrollerdb.txt"
sdl = {}
for line in io.open(os.path.join(src, "sdl", "gamecontrollerdb.txt"), encoding="utf-8", errors="replace"):
    if line.startswith("#") or "," not in line:
        continue
    guid, name = line.split(",")[:2]
    if not re.fullmatch(r"[0-9a-f]{32}", guid) or guid[4:8] != "0000" or guid[12:16] != "0000" or guid[20:24] != "0000":
        continue
    vid, pid = guid[10:12] + guid[8:10], guid[18:20] + guid[16:18]
    if vid == "0000":
        continue
    sdl.setdefault((vid, pid), set()).add(name.strip())
for (vid, pid), names in sorted(sdl.items()):
    if vid not in usb_makers and vid not in makers and vid in reg and len(reg[vid]) == 1:
        if any(shares(n, reg[vid][0]) for n in names):
            add_maker(vid, "controller", "sdl")
    mk = known_maker(vid)
    if not mk or (vid, pid) in usb_products or (vid, pid) in kb_products:
        continue
    fit = sorted(n for n in names if shares(n, mk))
    if not fit:
        continue
    name = clean(min(fit, key=len))
    # a controller name that only repeats the maker says nothing about the product
    if not name or not [x for x in toks(name) if x not in toks(mk)]:
        continue
    if (vid, pid) in products and SDL not in products[(vid, pid)][2]:
        products[(vid, pid)] = (products[(vid, pid)][0], "two sources", products[(vid, pid)][2] + " ; " + SDL)
    elif (vid, pid) not in products:
        products[(vid, pid)] = (name, "one source", SDL)

# ----- write -----
os.makedirs(out, exist_ok=True)
vids = sorted(set(makers) | set(v for v, _ in products))
lines = ["# vallum-pack devices 1"]
src_rows = ["code\tname\thow sure\tfrom"]
for vid in vids:
    name = makers[vid][0] if vid in makers else "-"
    lines.append("%s\t%s" % (vid, name))
    if vid in makers:
        src_rows.append("%s\t%s\t%s\t%s" % (vid, name, makers[vid][2], makers[vid][3]))
    for (v, pid) in sorted(p for p in products if p[0] == vid):
        pn, sure, where = products[(v, pid)]
        lines.append("\t%s\t%s" % (pid, pn))
        src_rows.append("%s:%s\t%s\t%s\t%s" % (v, pid, pn, sure, where))
io.open(os.path.join(out, "devices.txt"), "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
io.open(os.path.join(out, "devices-sources.tsv"), "w", encoding="utf-8", newline="\n").write("\n".join(src_rows) + "\n")

by_type, by_sure = {}, {}
for v in makers.values():
    by_type[v[1]] = by_type.get(v[1], 0) + 1
    by_sure[v[2]] = by_sure.get(v[2], 0) + 1
psure = {}
for p in products.values():
    psure[p[1]] = psure.get(p[1], 0) + 1
print("maker codes with a name:", len(makers), " maker codes in the file:", len(vids), " products:", len(products))
print("makers by type:", dict(sorted(by_type.items())))
print("makers by how sure:", by_sure)
print("products by how sure:", psure)
print("left out:", len(dropped))
for d in dropped:
    print("  ", d)
