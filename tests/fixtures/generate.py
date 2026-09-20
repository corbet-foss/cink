#!/usr/bin/env python3
"""Generate the binary image fixtures for the conformance vectors.

All fixtures are hand-constructed minimal files (no image libraries), so
every byte is deliberate and reproducible. Run from the repository root:

    python3 tests/fixtures/generate.py
"""
import struct
import sys
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return struct.pack(">I", len(payload)) + kind + payload + struct.pack(
        ">I", zlib.crc32(kind + payload) & 0xFFFFFFFF
    )


def write_png(path: Path, width: int, height: int) -> None:
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    raw = b"\x00" + b"\x80\x80\x80" * width
    idat = zlib.compress(b"".join(raw for _ in range(height)))
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", idat)
        + png_chunk(b"IEND", b"")
    )


def write_jpeg(path: Path, sof_marker: int, width: int, height: int) -> None:
    app0 = b"JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00"
    dqt = b"\x00" + bytes(64)
    sof = struct.pack(">HBHHBBBB", 11, 8, height, width, 1, 1, 0x11, 0)
    dht = b"\x00\x00"
    sos = struct.pack(">HBBBBBB", 8, 1, 1, 0, 0, 63, 0)
    path.write_bytes(
        b"\xff\xd8"
        + b"\xff\xe0" + struct.pack(">H", len(app0) + 2) + app0
        + b"\xff\xdb" + struct.pack(">H", len(dqt) + 2) + dqt
        + bytes((0xFF, sof_marker)) + sof
        + b"\xff\xc4" + struct.pack(">H", len(dht) + 2) + dht
        + b"\xff\xda" + sos + b"\x00"
        + b"\xff\xd9"
    )


def main() -> None:
    write_png(HERE / "pixel-1x1.png", 1, 1)
    write_png(HERE / "square-4x4.png", 4, 4)
    write_png(HERE / "rect-100x50.png", 100, 50)
    full = (HERE / "pixel-1x1.png").read_bytes()
    (HERE / "trunc-20b.png").write_bytes(full[:20])
    write_jpeg(HERE / "tiny-sof0.jpg", 0xC0, 1, 1)
    write_jpeg(HERE / "tiny-sof2.jpg", 0xC2, 1, 1)
    (HERE / "icon.svg").write_bytes(
        b'<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"></svg>'
    )
    (HERE / "anim.gif").write_bytes(b"GIF89a\x01\x00\x01\x00\x80\x00\x00")
    (HERE / "loss.webp").write_bytes(b"RIFF\x24\x00\x00\x00WEBPVP8 ")
    (HERE / "garbage.bin").write_bytes(b"not an image at all")
    (HERE / "empty.bin").write_bytes(b"")
    print("fixtures written:", sorted(p.name for p in HERE.iterdir()))


if __name__ == "__main__":
    sys.exit(main())
