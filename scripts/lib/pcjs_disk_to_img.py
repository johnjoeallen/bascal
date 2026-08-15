#!/usr/bin/env python3
"""Reconstruct a raw floppy disk image from a PCjs JSON disk image.

PCjs (https://www.pcjs.org) publishes vintage software as JSON-encoded disk
images rather than raw binary images, one field of many being a per-track,
per-sector array of 32-bit little-endian words describing that sector's
bytes. A sector whose trailing words are all identical to its last word may
have that run compressed away -- the decoder is expected to repeat the final
word to fill out the sector to its declared length ('l').

This script performs the inverse of that encoding: given a PCjs disk JSON
file, it writes out the equivalent raw .img file, laid out in the standard
cylinder-major, head-minor, sector-minor CHS order that real floppy
controllers (and dosbox-x's IMGMOUNT) expect.

Usage:
    pcjs_disk_to_img.py <input.json> <output.img>

Prints the disk image's declared MD5 hash and byte size (as published in the
JSON's own imageInfo block) to stdout on success, so callers can verify the
reconstructed image against PCjs's own metadata without computing their own
reference hash.
"""
import json
import struct
import sys


def convert(json_path, img_path):
    with open(json_path, "r", encoding="utf-8") as f:
        disk = json.load(f)

    info = disk["imageInfo"]
    cylinders = info["cylinders"]
    heads = info["heads"]
    disk_data = disk["diskData"]

    if len(disk_data) != cylinders:
        raise ValueError(
            f"expected {cylinders} cylinders of sector data, found {len(disk_data)}"
        )

    out = bytearray()
    for cylinder in range(cylinders):
        track = disk_data[cylinder]
        if len(track) != heads:
            raise ValueError(
                f"cylinder {cylinder}: expected {heads} heads, found {len(track)}"
            )
        for head in range(heads):
            sectors = sorted(track[head], key=lambda sector: sector["s"])
            for sector in sectors:
                length = sector["l"]
                words = list(sector["d"])
                last_word = words[-1] if words else 0
                # PCjs omits a sector's trailing words when they all equal
                # the final word -- restore them before packing to bytes.
                while len(words) * 4 < length:
                    words.append(last_word)
                packed = bytearray()
                for word in words:
                    packed += struct.pack("<i", word)
                out += packed[:length]

    with open(img_path, "wb") as f:
        f.write(out)

    return info["hash"], info["diskSize"], len(out)


def main(argv):
    if len(argv) != 3:
        print(f"usage: {argv[0]} <input.json> <output.img>", file=sys.stderr)
        return 2

    published_hash, published_size, actual_size = convert(argv[1], argv[2])
    if actual_size != published_size:
        print(
            f"error: reconstructed image is {actual_size} bytes, "
            f"but the JSON declares diskSize={published_size}",
            file=sys.stderr,
        )
        return 1

    print(f"{published_hash} {published_size}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
