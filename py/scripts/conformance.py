"""Run canonical vectors against either the source port or an installed wheel."""
import base64
import importlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
if "--installed" not in sys.argv:
    sys.path.insert(0, str(ROOT / "py"))
api = importlib.import_module("cink")

ARGUMENTS = {
    "region_parse": ["input"], "region_uses_comma": ["input"],
    "salutation_last_name": ["input"], "salutation_honorific": ["input"],
    "salutation_titles": ["input"], "salutation_surname": ["input"],
    "de_salutation": ["input", "region"],
    "recipient_salutation_warning": ["location", "input"],
    "de_honorific_warning": ["location", "input"],
    "closing": ["locale", "override"], "available_locales": [],
    "long_date": ["locale", "year", "month", "day"],
    "medium_date": ["locale", "year", "month", "day"],
    "short_date": ["locale", "year", "month", "day"],
    "month_year": ["locale", "year", "month"], "is_supported": ["locale"],
    "is_valid_date": ["year", "month", "day"],
    "default_max_pixels": [], "supported_formats": [], "normalize": ["input"],
    "exceeds_limits": ["image", "max_pixels"], "scale_to_fit": ["image", "max_pixels"],
    "signature_size": ["image", "height_pt", "max_width_pt"],
    "image_snippet": ["path", "height_pt", "width_pt"],
    "resolve_locale": ["language", "location"],
    "opening": ["locale", "person", "override"],
    "subject": ["locale", "title", "prefix_override"],
    "salutation": ["locale", "person"], "apply_ortho": ["locale", "text"],
    "warnings": ["location", "locale", "person"],
}

files = sorted((ROOT / "tests/vectors").glob("*.json"))
assert files, "No conformance vectors"
count = 0
for file in files:
    for vector in json.loads(file.read_text(encoding="utf-8")):
        name = vector["fn"]
        values = dict(vector)
        if "fixture" in vector:
            values["input"] = base64.b64encode((ROOT / "tests/fixtures" / vector["fixture"]).read_bytes()).decode("ascii")
            if name != "normalize":
                values["image"] = api.normalize(values["input"])
        fn = getattr(api, "parse_region" if name == "region_parse" else name)
        actual = fn(*(values.get(key) for key in ARGUMENTS[name]))
        if name == "normalize" and actual is not None:
            actual = {key: getattr(actual, key) for key in ("mime", "width", "height")}
        if isinstance(actual, tuple):
            actual = list(actual)
        if actual != vector["expected"]:
            raise AssertionError(f"{file.name} :: {vector['name']}: {actual!r} != {vector['expected']!r}")
        count += 1
image = api.DecodedImage("png", 1, 1, b"")
for invalid in (float("nan"), float("inf"), -float("inf"), 0, -1, True, 10**400):
    assert api.signature_size(image, invalid) is None
    assert api.signature_size(image, 20, invalid) is None
for invalid_dimension in (0, -1, 1.5, True, 0x100000000):
    assert api.signature_size(api.DecodedImage("png", invalid_dimension, 1, b""), 20) is None
assert api.signature_size(api.DecodedImage("png", 2, 1, b""), sys.float_info.max, 30) == (30, 15)
assert api.signature_size(api.DecodedImage("png", 1, 2, b""), float.fromhex("0x0.0000000000001p-1022")) is None
print(f"Python cink: {count} vectors across {len(files)} files and numeric boundary checks passed")
