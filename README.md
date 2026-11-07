# Mati Mati · coniugatio

A website where people can learn about indigenous languages and conjugate their verbs, built with the Mati Mati brand.

Live site: https://nojsuon.github.io/coniugatio/

## How it works

Everything is static, so it runs on GitHub Pages:

- **`crates/core`**: the conjugation engine in Rust (clitics, root changes, plural).
- **`crates/wasm`**: compiles the engine to WebAssembly so it runs in the browser.
- **`crates/site`**: builds the site into `dist/`, and can also serve it locally.
- **`data/languages/*.json`**: one file per language (text, orthography, grammar, conjugation tables).
- **`static/`**: HTML templates, CSS (Mati Mati design tokens) and JavaScript.

## Run it locally

You need Rust, plus these two tools (one-time setup):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100 --locked
```

Then build and serve the site at http://127.0.0.1:3000/:

```bash
cargo run -p coniugatio-site -- --serve
```

Run the tests:

```bash
cargo test --workspace --exclude coniugatio-wasm
```

## Publishing

Every push to `main` builds the site and deploys it to GitHub Pages (`.github/workflows/pages.yml`).

## Adding a language

1. Add `data/languages/<slug>.json` with `"status": "soon"`; a card appears on the home page.
2. When its conjugation data is ready, add a `conjugation` block and set `"status": "available"`.

## Sources

- Otomí (hñähñu): Hernández Cruz, Victoria Torquemada y Sinclair Crawford, *Diccionario del hñähñu (otomí) del Valle del Mezquital*, Instituto Lingüístico de Verano, 2010.
- Maya itza': Charles Andrew Hofling con Fernando Tesucún, *Itzaj Maya Grammar*, University of Utah Press, 2000; *Jiilt'an Maya Itza: Vocabulario Itza'*, Academia de Lenguas Mayas de Guatemala, 2001.

All text in an indigenous language should be checked with speakers before publishing.

Each language's `conjugation` block names its `engine` (`otomi` or `itza`), which selects the rules in `crates/core`.
