"""Validate and size handwritten signature images for correspondence."""

import base64
import binascii
import math
import struct
from dataclasses import dataclass
from typing import Literal

from ._tables import TABLES

_DEFAULTS = TABLES["defaults"]
_PNG = b"\x89PNG\r\n\x1a\n"
_SOF = {0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf}
__all__ = ["default_max_pixels", "supported_formats", "normalize", "exceeds_limits",
           "scale_to_fit", "signature_size", "image_snippet"]


@dataclass(frozen=True)
class DecodedImage:
    """Image format, optional raster dimensions, and original encoded bytes."""

    mime: Literal["png", "jpeg", "svg"]
    width: int | None
    height: int | None
    bytes: bytes


def default_max_pixels() -> int:
    """Default raster pixel budget."""
    return _DEFAULTS["max_pixels"]


def supported_formats() -> list[str]:
    """Accepted image formats."""
    return sorted(_DEFAULTS["formats"])


def _jpeg_dimensions(data: bytes) -> tuple[int, int] | None:
    pos = 2
    while pos < len(data):
        if data[pos] != 0xff:
            return None
        while pos < len(data) and data[pos] == 0xff:
            pos += 1
        if pos >= len(data):
            return None
        marker = data[pos]
        pos += 1
        if marker == 1:
            continue
        if marker == 0 or 0xd0 <= marker <= 0xda:
            return None
        if pos + 2 > len(data):
            return None
        length = struct.unpack_from(">H", data, pos)[0]
        if length < 2 or length > len(data) - pos:
            return None
        if marker in _SOF:
            if length < 8:
                return None
            components = data[pos + 7]
            if not components or length != 8 + 3 * components:
                return None
            height, width = struct.unpack_from(">HH", data, pos + 3)
            return (width, height) if width and height else None
        pos += length
    return None


def normalize(value: str) -> DecodedImage | None:
    """Read PNG/JPEG headers or pass SVG through from canonical base64/a data URL.

    Required padding must be present; whitespace and nonzero unused bits are rejected.
    Header recognition does not fully decode a raster or sanitize SVG markup.
    """
    payload = value
    if value.startswith("data:") and "," in value:
        meta, payload = value.split(",", 1)
        if not meta.endswith(";base64"):
            return None
    if not payload:
        return None
    try:
        data = base64.b64decode(payload, validate=True)
        if base64.b64encode(data).decode("ascii") != payload:
            return None
    except (ValueError, binascii.Error):
        return None
    if data.startswith(_PNG):
        if len(data) < 33 or data[8:16] != b"\x00\x00\x00\x0dIHDR":
            return None
        width, height = struct.unpack_from(">II", data, 16)
        if not 0 < width <= 0x7fffffff or not 0 < height <= 0x7fffffff:
            return None
        depth, color, compression, filtering, interlace = data[24:29]
        valid_depths = {0: (1, 2, 4, 8, 16), 2: (8, 16), 3: (1, 2, 4, 8), 4: (8, 16), 6: (8, 16)}
        if depth not in valid_depths.get(color, ()) or compression != 0 or filtering != 0 or interlace > 1:
            return None
        return DecodedImage("png", width, height, data)
    if data.startswith(b"\xff\xd8"):
        dims = _jpeg_dimensions(data)
        return DecodedImage("jpeg", *dims, data) if dims else None
    if data.lstrip(b" \t\n\r\f").startswith(b"<svg"):
        return DecodedImage("svg", None, None, data)
    return None


def exceeds_limits(image: DecodedImage, max_pixels: int) -> bool:
    """Whether raster dimensions exceed a pixel budget."""
    return image.width is not None and image.height is not None and image.width * image.height > max_pixels


def scale_to_fit(image: DecodedImage, max_pixels: int) -> float:
    """Scale factor at most one; callers perform any resampling."""
    if image.width is None or image.height is None or not exceeds_limits(image, max_pixels):
        return 1.0
    return math.sqrt(max_pixels / (image.width * image.height))


def signature_size(image: DecodedImage, height_pt: float, max_width_pt: float | None = None) -> tuple[float, float] | None:
    """Aspect-preserving point size, or None for invalid/nonrepresentable sizes."""
    if (type(image.width) is not int or type(image.height) is not int
            or not 0 < image.width <= 0xffffffff or not 0 < image.height <= 0xffffffff
            or type(height_pt) not in (int, float)
            or (max_width_pt is not None and type(max_width_pt) not in (int, float))):
        return None
    try:
        height_pt = float(height_pt)
        if max_width_pt is not None:
            max_width_pt = float(max_width_pt)
    except OverflowError:
        return None
    if (not math.isfinite(height_pt) or height_pt <= 0
            or (max_width_pt is not None and (not math.isfinite(max_width_pt) or max_width_pt <= 0))):
        return None
    ratio = image.width / image.height
    width = height_pt * ratio
    if max_width_pt is not None and width > max_width_pt:
        width, height_pt = max_width_pt, max_width_pt / ratio
    return (width, height_pt) if math.isfinite(width) and math.isfinite(height_pt) and width > 0 and height_pt > 0 else None


def image_snippet(path: str, height_pt: float, width_pt: float | None = None) -> str:
    """A Typst image call; placement remains the caller's responsibility."""
    escapes = {'"': '\\"', "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t"}
    safe = "".join(escapes.get(char, f"\\u{{{ord(char):x}}}" if ord(char) < 32 or 127 <= ord(char) <= 159 else char) for char in path)
    height = format(height_pt, ".15g")
    width = "" if width_pt is None else f", width: {format(width_pt, '.15g')}pt"
    return f'#image("{safe}", height: {height}pt{width})'
