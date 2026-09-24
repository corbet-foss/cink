# Changelog

All notable changes to `cink` are documented here. The project follows
Semantic Versioning.

## 0.2.1 - 2026-09-24

- Repository moved to github.com/corbet-foss/cink; registry metadata points there.
- Released from a single tag through CI (crates.io and JSR trusted publishing).
- Drop the duplicate `LICENSES/LGPL-3.0-only WITH LGPL-3.0-linking-exception.txt`
  (identical to `LGPL-3.0-linking-exception.txt`); JSR rejects paths with spaces.

## 0.2.0 - 2026-09-13

- License this new release line under LGPL-3.0-only WITH LGPL-3.0-linking-exception across Cargo, npm, JSR,
  Python and Typst, with the complete LGPL and incorporated GPL notices.
- Keep runtime behavior, correspondence tables and dependency versions unchanged.

## 0.1.4 - 2026-09-11

- Include consistent license texts, notices, and package metadata.
- Prepare the Typst package for Universe submission; runtime behavior is unchanged.

## 0.1.3 - 2026-09-09

- Reject malformed JPEG segment framing and invalid or non-finite signature sizes.
- Verify invalid input consistently across Rust, JavaScript and Python.
- Reuse packed distributions and run selected checks through Crow.

## 0.1.2 - 2026-09-09

- Ship compiled ESM and CommonJS, declaration files, and a standalone browser module.
- Add a Python distribution with shared-vector conformance and a JSON CLI.
- Add JSR packaging, installed-artifact tests, and complete registry license files.
- Clarify scope, examples, installation options, and family links on the product page.

## 0.1.1 - 2026-09-06

- Add the `supportedFormats` parity export (missing from the TypeScript
  side in 0.1.0). No behavior change.

## 0.1.0 - 2026-09-06

- Initial release: signature-image validation (PNG/JPEG with header
  dimensions, SVG passthrough; WebP/GIF/corrupt rejected), oversize
  reporting with fit scale, point-size resolution with max-width clamp,
  and Typst snippet emission — as a Rust crate, a pure-TypeScript
  package, and a Typst module sharing one vector suite.
