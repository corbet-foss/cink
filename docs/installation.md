# Installation and distribution

The main branch prepares 0.2.0 under LGPL-3.0-only WITH LGPL-3.0-linking-exception. It has not been published
to registries. The existing releases documented below keep their original
license grants; this change does not replace their artifacts.

## JavaScript and Rust

Use Cargo for Rust and npm, pnpm, Yarn, or Bun for JavaScript. These JavaScript
package managers share the npm registry; each consumes the same package.
Deno can use `npm:@corbet-labs/cink`. The browser export bundles runtime
dependencies and needs no import map. The declared Rust minimum is 1.94. Release checks use the worker's current stable compiler; a separate minimum-version check is required to
verify that lower bound.

## Python and the command line

The pure Python package requires Python 3.10+ and is published on
[PyPI](https://pypi.org/project/cink/0.2.0/). Install this release with pip
or uv in your Python environment:

```sh
python -m pip install cink==0.2.0
```

```sh
uv pip install cink==0.2.0
```

For a uv project, `uv add cink==0.2.0` adds the package to your dependencies.

After installation, functions can be called from Python or through either CLI
entrypoint:

```sh
cink --help
python -m cink --help
```

The CLI takes a function name and a JSON array of positional arguments or an
object of keyword arguments. Use `-` to read arguments from stdin. It writes
JSON to stdout; errors use stderr and a nonzero exit status. Image bytes in
the `normalize` result are base64 strings.

For an isolated CLI environment, use `pipx install cink==0.2.0` or
`uv tool install cink==0.2.0`.

The wheel contains no native extensions and is platform independent. Release
evidence records the Python version and operating system actually exercised.
Verified wheels and source distributions are also attached to the
[GitHub release](https://github.com/corbet-foss/cink/releases/tag/v0.2.0).

## JSR

The 0.2.0 release targets
[`@corbet-labs/cink`](https://jsr.io/@corbet-labs/cink@0.2.0):

```sh
deno add jsr:@corbet-labs/cink@0.2.0
```

## Typst

Download `cink-0.2.0-typst.tar.gz` from the matching GitHub release and
extract its contents into `typst/packages/local/cink/0.2.0` under your
[Typst data directory](https://github.com/typst/packages#local-packages):

| System | Data directory |
| --- | --- |
| Linux | `$XDG_DATA_HOME`, or `~/.local/share` |
| macOS | `~/Library/Application Support` |
| Windows | `%APPDATA%` |

```typst
#import "@local/cink:0.2.0": *
```

The archive includes its manifest, tables, source, and licenses. CI compiles
an example against a fresh installation of the actual archive with Typst 0.15.
For the Typst web app, upload the extracted files and import the entrypoint
listed in `typst.toml` by its relative path.

For signature images, load bytes in your document so paths resolve from the
document directory:

```typst
#signature-image(read("signature.svg", encoding: none), 24)
```

The Typst image helper places an image. Header parsing, pixel limits, and
sizing calculations are available in the Rust, JavaScript, and Python APIs.

The Apache-2.0 release is submitted to Typst Universe in
[PR #5820](https://github.com/typst/packages/pull/5820), alongside the other
correspondence components. The prepared preview examples passed on Typst 0.15.0.
The listing awaits review and publication; use the local archive above until
the preview package is available.

## System package managers

These are language libraries, with a portable Python CLI. Homebrew, APT, RPM,
WinGet, Chocolatey, and Scoop are not additional registries for importing a
Rust crate or JavaScript module. Use their Python or Node runtime and the
language package manager above. No native system-package listing is claimed.
