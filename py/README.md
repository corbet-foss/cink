# cink

**Prepare handwritten signature images for correspondence.**

[![crates.io](https://img.shields.io/crates/v/cink.svg)](https://crates.io/crates/cink) [![npm](https://img.shields.io/npm/v/@corbet-labs/cink.svg)](https://www.npmjs.com/package/@corbet-labs/cink) [![PyPI](https://img.shields.io/pypi/v/cink.svg)](https://pypi.org/project/cink/) [![Rust API](https://docs.rs/cink/badge.svg)](https://docs.rs/cink)

Recognize PNG, JPEG, and SVG input, inspect raster dimensions, calculate an aspect-preserving size, and generate a Typst image call. Runs without an image-decoding library.

```js
import { imageSnippet } from '@corbet-labs/cink';

imageSnippet('/signature.png', 31.5);
// #image("/signature.png", height: 31.5pt)
```

## Install

| Environment | Command |
| --- | --- |
| Rust / Cargo | `cargo add cink` |
| Python / pip | `python -m pip install cink` |
| Python / uv | `uv add cink` |
| Node.js / npm | `npm install @corbet-labs/cink` |
| pnpm | `pnpm add @corbet-labs/cink` |
| Yarn | `yarn add @corbet-labs/cink` |
| Bun | `bun add @corbet-labs/cink` |
| Deno | `deno add npm:@corbet-labs/cink` |

The 0.1.4 JavaScript distribution includes compiled ESM, CommonJS,
TypeScript declarations, and a standalone browser module. Node.js 20+ is
supported; no TypeScript loader is required.

```js
// CommonJS
const { imageSnippet } = require('@corbet-labs/cink');
```

```html
<script type="module">
  import { imageSnippet } from 'https://cdn.jsdelivr.net/npm/@corbet-labs/cink@0.1.4/dist/browser.js';
  console.log(imageSnippet('/signature.png', 31.5));
</script>
```

Python 3.10+ packages are available on [PyPI](https://pypi.org/project/cink/).
See the [installation guide](https://github.com/corbet-foss/cink/blob/main/docs/installation.md)
for CLI commands and other distribution options.
JSR publication and Typst availability are listed there explicitly.

## Rust

```rust
use cink::image_snippet;

assert_eq!(image_snippet("/signature.png", 31.5, None), "#image(\"/signature.png\", height: 31.5pt)");
```

## Python

```python
from cink import image_snippet

assert image_snippet("/signature.png", 31.5) == '#image("/signature.png", height: 31.5pt)'
```

## API

| JavaScript / Python or Rust | Purpose |
| --- | --- |
| `normalize` | Recognize a base64 image or data URL |
| `exceedsLimits` / `exceeds_limits` | Check a raster pixel budget |
| `scaleToFit` / `scale_to_fit` | Calculate a downscaling factor |
| `signatureSize` / `signature_size` | Fit dimensions to a point size |
| `imageSnippet` / `image_snippet` | Generate a Typst image call |

Base64 input uses the standard alphabet, required padding, and canonical trailing bits; embedded whitespace is rejected.

PNG and JPEG dimensions are read from headers; this is not a full raster integrity check. SVG passes through without sanitization or known dimensions. Unrecognized headers and unsupported formats return `null`/`None`. Nonpositive, nonfinite, and unrepresentable requested sizes return `null`/`None`. Resampling and placement belong to the caller. These are handwriting images, not cryptographic or electronic signatures.

## Correspondence family

| Library | Responsibility |
| --- | --- |
| [cletter](https://github.com/corbet-foss/cletter) | Compose the correspondence helpers |
| [cgreet](https://github.com/corbet-foss/cgreet) | German salutations and titles |
| [cfarewell](https://github.com/corbet-foss/cfarewell) | Locale-specific closings |
| [cdate](https://github.com/corbet-foss/cdate) | Calendar-date formatting |
| [cink](https://github.com/corbet-foss/cink) | Handwritten signature images |


## Development

Behavior is defined by [the signature limits](https://github.com/corbet-foss/cink/tree/main/tables)
and [shared conformance vectors](https://github.com/corbet-foss/cink/tree/main/tests/vectors).
Rust, JavaScript, and Python run the same vectors. Selected CI checks exercise
installed JavaScript tarballs, Python wheels and command-line entrypoints, and
Typst packages. Release validation records the actual runtime and platform;
Linux results do not establish native Windows or macOS coverage.
All five Rust crates forbid unsafe code in their own source.

See [the release guide](https://github.com/corbet-foss/cink/blob/main/docs/releasing.md)
for generation, verification, and publication commands.

## License

Copyright 2026 Julian Y. Richard Corbet. The 0.2.1 release line is licensed
under [LGPL-3.0-only](https://github.com/corbet-foss/cink/blob/main/LICENSES/LGPL-3.0-only.txt)
[WITH LGPL-3.0-linking-exception](https://github.com/corbet-foss/cink/blob/main/LICENSES/LGPL-3.0-linking-exception.txt),
with the incorporated [GPL version 3](https://github.com/corbet-foss/cink/blob/main/LICENSES/GPL-3.0-only.txt).
Combined works may link statically or dynamically without relinking duties;
library modifications stay LGPL. Applications can use a different license
subject to the LGPL's conditions.
Version 0.1.4 retains Apache-2.0. The installation examples above refer to those available
releases; 0.2.1 is published to registries.

See the [licensing notes](https://github.com/corbet-foss/cink/blob/main/LICENSE.md) for distribution conditions and retained notices.
Contributions use the [Contributor License Agreement](https://github.com/corbet-foss/cink/blob/main/CLA.md).
