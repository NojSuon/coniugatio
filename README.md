# Mati Mati · coniugatio

A website where people can learn about indigenous languages and conjugate their verbs, built with the Mati Mati brand.

Live site: https://nojsuon.github.io/coniugatio/ (in English: https://nojsuon.github.io/coniugatio/en/)

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

## English pages

Every page also exists in English under `/en/` (`/coniugatio/en/otomi/`…), with a button at the top right to switch between the two.

- **Text:** `data/languages/en/<slug>.json` holds only the translated text and is merged over the Spanish file at build time. Objects merge key by key, lists of tenses, persons and verbs merge by `id` or `entry`, and other lists merge by position, so keep the same order.
- **Equivalents:** on English pages each tense's `es` spec and each verb's `es` meaning are given in English, and `crates/core/src/english.rs` conjugates them ("I was singing"). Constructions use English auxiliaries: `{ "tense": "present", "aux": "need", "link": "to" }`.
- **Checks:** the build warns when a listed verb has a form without its English equivalent.

## Adding a language

1. Add `data/languages/<slug>.json` with `"status": "soon"`; a card appears on the home page.
2. When its conjugation data is ready, add a `conjugation` block and set `"status": "available"`.

## Sources

- Otomí (hñähñu): Hernández Cruz, Victoria Torquemada y Sinclair Crawford, *Diccionario del hñähñu (otomí) del Valle del Mezquital*, Instituto Lingüístico de Verano, 2010.
- Náhuatl (maseuallâjtolli): Thelma D. Sullivan, *Compendio de la gramática náhuatl* (UNAM); David Charles Wright Carr, *Lectura del náhuatl* (INALI, 2016); spelling: Tlâjkuilolpamitl (SEP).
- P'urhépecha: Ma. Guadalupe Hernández Dimas, Roberto Fabián Luis y Sereno Coló, *Curso de lengua p'urhépecha* (apuntes de la clase de Fernando Nava, IIA-UNAM, 1998–1999); Alberto Medina Pérez, *Guía lingüística p'urhépecha* (UMSNH, 2006); Paul P. de Wolf, *Curso básico del tarasco hablado* (El Colegio de Michoacán).
- Maya yucateco (maayat'aan): Fidencio Briceño Chel, *Los verbos del maya yucateco actual* (INALI, 2021); INALI, *Normas de escritura para la lengua maya* (2014); also the *Diccionario maya de uso cotidiano* (Quintana Roo) and Bolles, *A Grammar of the Yucatecan Mayan Language*.
- Mopan: Charles Andrew Hofling, *Mopan Maya–Spanish–English Dictionary* (University of Utah Press, 2011); Academia de Lenguas Mayas de Guatemala, *Muuch' t'an mopan* (2003); *Diccionario bilingüe maya mopán y español*.
- Zoque (tsunitsame), Copainalá: Ralph y Mary Engel, *Diccionario zoque de Copainalá* (ILV, 1981), con su gramática; INALI, *Norma de escritura de la lengua otetzame (zoque)*, escrita con ts en lugar de tz; also Engel, *Diccionario zoque de Francisco León* (ILV, 1987).
- Maya itza': Charles Andrew Hofling con Fernando Tesucún, *Itzaj Maya Grammar*, University of Utah Press, 2000; *Jiilt'an Maya Itza: Vocabulario Itza'*, Academia de Lenguas Mayas de Guatemala, 2001.

All text in an indigenous language should be checked with speakers before publishing.

Each language's `conjugation` block names its `engine` (`otomi`, `itza`, `nahuatl`, `purhepecha`, `yucatec`, `mopan` or `zoque`), which selects the rules in `crates/core`.
