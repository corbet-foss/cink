# tables — canonical signature limits

`defaults.json` holds the only numbers in the library: the oversize pixel
budget and the accepted formats. Everything else is byte-level format
rules, identical in every port:

- **PNG**: 8-byte magic, then `IHDR` with length 13; width/height are
  big-endian `u32` at bytes 16..24. Nonzero, or the file is corrupt.
- **JPEG**: `SOI` (`FF D8`), then length-prefixed segments; the first Start
  Of Frame marker (`C0–C3, C5–C7, C9–CB, CD–CF`) carries precision, height,
  width. Stops at `SOS`/`EOI`. Fill bytes tolerated.
- **SVG**: leading ASCII whitespace, then `<svg`. Passed through with
  unknown dimensions — the caller sizes it explicitly.
- **Anything else** (WebP, GIF, corrupt, empty, non-base64): rejected.

`tests/vectors/*.json` with fixtures in `tests/fixtures/` (built by the
checked-in `generate.py`, no image libraries) is the executable contract.
