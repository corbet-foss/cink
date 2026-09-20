// Partial cross-port conformance for `typst/ink.typ`.
//
// The binary vectors in `tests/vectors/*.json` (base64 fixtures, pixel
// budgets, point sizes) cannot run in Typst: there is no base64 decoder or
// binary reader in the Typst standard library, and `signature-image`
// returns laid-out content rather than the `#image` call string the other
// ports assert on. Those vectors stay Rust/JS/Python-only by design.
// What Typst can share — the table-driven budget and format entries — is
// asserted here.
//
// Compile from the repository root:
//   typst compile --root . tests/conformance.typ /tmp/cink-conformance.pdf
#import "../typst/ink.typ" as api

#assert.eq(api.default-max-pixels(), 1600, message: "default pixel budget")
#assert.eq(api.supported-formats(), ("jpeg", "png", "svg"), message: "supported formats")

Typst conformance green: 2 table assertions (binary vectors are Rust/JS/Python-only).
