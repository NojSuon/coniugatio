//! Builds the Mati Mati site into `dist/` as plain static files, ready for
//! GitHub Pages:
//!
//! ```text
//! dist/index.html            home page
//! dist/<slug>/index.html     one page per language (/otomi/, /itza/, ...)
//! dist/404.html
//! dist/data/languages.json   home page cards
//! dist/data/<slug>.json      everything about one language
//! dist/en/...                the same pages in English, with
//! dist/data/en/...           data/languages/en/<slug>.json merged over
//!                            each language (only the text that changes)
//! dist/static/...            css, js, images
//! dist/pkg/...               the conjugator compiled to WebAssembly
//! ```
//!
//! Usage:
//!   cargo run -p coniugatio-site                 build with base "/"
//!   cargo run -p coniugatio-site -- --serve      build and serve on :3000
//!   cargo run -p coniugatio-site -- --base coniugatio --serve
//!       build as on GitHub Pages (site under /coniugatio/). Write the base
//!       without a leading slash: Git Bash on Windows rewrites "/x" into a
//!       Windows path.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use coniugatio_core::{conjugate::Options as ConjugateOptions, Engine, Summary};
use serde_json::Value;

struct Options {
    base: String,
    out: PathBuf,
    wasm: bool,
    serve: Option<u16>,
}

fn parse_args(root: &Path) -> Options {
    let mut opts = Options { base: "/".into(), out: root.join("dist"), wasm: true, serve: None };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base" => {
                let mut base = args.next().expect("--base needs a value, e.g. /coniugatio/");
                if !base.starts_with('/') {
                    base.insert(0, '/');
                }
                if !base.ends_with('/') {
                    base.push('/');
                }
                opts.base = base;
            }
            "--out" => opts.out = PathBuf::from(args.next().expect("--out needs a directory")),
            "--no-wasm" => opts.wasm = false,
            "--serve" => opts.serve = Some(3000),
            "--port" => opts.serve = Some(args.next().and_then(|p| p.parse().ok()).expect("--port needs a number")),
            other => panic!("unknown argument: {other}"),
        }
    }
    opts
}

fn main() {
    // Workspace root (crates/site/../..). Not canonicalized: on Windows that
    // adds a \\?\ prefix that some tools reject.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let opts = parse_args(&root);
    if let Err(e) = build(&root, &opts) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    println!("Site built in {}", opts.out.display());

    if let Some(port) = opts.serve {
        serve(&opts.out, &opts.base, port);
    }
}

struct Language {
    summary: Summary,
    raw: Value,
    /// The same data with the English text merged in.
    en: Value,
    summary_en: Summary,
}

/// Merges an English overlay into a language's data. Objects merge key by
/// key. Arrays of objects with an "id" or "entry" (tenses, persons, types,
/// lexicon) merge by that key; other arrays merge position by position, so
/// an overlay only lists the text it translates, in the same order.
fn merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                // An "es" spec is replaced whole: merging it would keep a
                // Spanish auxiliary ("acabar") in an English construction.
                if k == "es" {
                    b.insert(k.clone(), v.clone());
                    continue;
                }
                match b.get_mut(k) {
                    Some(slot) => merge(slot, v),
                    None => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (Value::Array(b), Value::Array(o)) => {
            let key = |v: &Value| {
                ["id", "entry"]
                    .iter()
                    .find_map(|k| v.get(*k).and_then(Value::as_str).map(|s| (*k, s.to_string())))
            };
            for (i, v) in o.iter().enumerate() {
                let by_key = key(v).and_then(|(k, id)| {
                    b.iter().position(|x| x.get(k).and_then(Value::as_str) == Some(id.as_str()))
                });
                match by_key.or_else(|| key(v).is_none().then_some(i).filter(|&i| i < b.len())) {
                    Some(j) => merge(&mut b[j], v),
                    None => b.push(v.clone()),
                }
            }
        }
        (b, o) => *b = o.clone(),
    }
}

fn load_languages(dir: &Path) -> Result<Vec<Language>, String> {
    let mut list = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let ctx = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        let raw: Value = serde_json::from_str(&fs::read_to_string(&path).map_err(|e| ctx(&e))?).map_err(|e| ctx(&e))?;
        let summary: Summary = serde_json::from_value(raw.clone()).map_err(|e| ctx(&e))?;
        let mut en = raw.clone();
        let en_path = dir.join("en").join(path.file_name().unwrap());
        if en_path.exists() {
            let over: Value = serde_json::from_str(&fs::read_to_string(&en_path).map_err(|e| ctx(&e))?)
                .map_err(|e| format!("{}: {e}", en_path.display()))?;
            merge(&mut en, &over);
        }
        let summary_en: Summary = serde_json::from_value(en.clone()).map_err(|e| ctx(&e))?;

        // Catch data mistakes at build time rather than in the browser.
        if let Some(c) = raw.get("conjugation") {
            let conj: Engine = serde_json::from_value(c.clone()).map_err(|e| ctx(&e))?;
            for word in conj.lexicon_words() {
                conj.conjugate(&word).map_err(|e| ctx(&format!("lexicon verb {word:?}: {e}")))?;
            }
            // The English pages: every form of a lexicon verb should have
            // its English equivalent.
            let conj_en: Engine = serde_json::from_value(en["conjugation"].clone()).map_err(|e| ctx(&e))?;
            let opts = ConjugateOptions { lang: Some("en".into()), ..Default::default() };
            for word in conj_en.lexicon_words() {
                let r = conj_en.conjugate_with(&word, &opts).map_err(|e| ctx(&format!("lexicon verb {word:?}: {e}")))?;
                let missing: Vec<&str> = r
                    .tables
                    .iter()
                    .filter(|t| t.rows.iter().any(|row| !row.singular.is_empty() && row.es_singular.is_none()))
                    .map(|t| t.id.as_str())
                    .collect();
                if !missing.is_empty() {
                    eprintln!("warning: {}: no English for {word:?} in {missing:?}", path.display());
                }
            }
        } else if summary.status == "available" {
            return Err(ctx(&"status is \"available\" but there is no \"conjugation\" block"));
        }
        list.push(Language { summary, raw, en, summary_en });
    }
    list.sort_by_key(|l| l.summary.order);
    Ok(list)
}

fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

/// Copies `from` into `to`, skipping the HTML templates.
fn copy_assets(from: &Path, to: &Path) -> Result<(), String> {
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_assets(&path, &target)?;
        } else if path.extension().and_then(|e| e.to_str()) != Some("html") {
            write(&target, fs::read(&path).map_err(|e| e.to_string())?)?;
        }
    }
    Ok(())
}

fn render(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |acc, (k, v)| acc.replace(&format!("{{{{{k}}}}}"), v))
}

fn build(root: &Path, opts: &Options) -> Result<(), String> {
    let out = &opts.out;
    if out.exists() {
        fs::remove_dir_all(out).map_err(|e| format!("cleaning {}: {e}", out.display()))?;
    }

    let languages = load_languages(&root.join("data/languages"))?;
    let summaries: Vec<&Summary> = languages.iter().map(|l| &l.summary).collect();
    write(&out.join("data/languages.json"), serde_json::to_string(&summaries).unwrap())?;
    for lang in &languages {
        write(&out.join(format!("data/{}.json", lang.summary.slug)), serde_json::to_string(&lang.raw).unwrap())?;
        write(&out.join(format!("data/en/{}.json", lang.summary.slug)), serde_json::to_string(&lang.en).unwrap())?;
    }
    let summaries_en: Vec<&Summary> = languages.iter().map(|l| &l.summary_en).collect();
    write(&out.join("data/en/languages.json"), serde_json::to_string(&summaries_en).unwrap())?;

    let templates = root.join("static");
    copy_assets(&templates, &out.join("static"))?;

    let read = |name: &str| fs::read_to_string(templates.join(name)).map_err(|e| format!("{name}: {e}"));
    let base = opts.base.as_str();
    write(&out.join("index.html"), render(&read("index.html")?, &[("base", base)]))?;
    write(&out.join("404.html"), render(&read("404.html")?, &[("base", base)]))?;
    write(&out.join("en/index.html"), render(&read("en/index.html")?, &[("base", base)]))?;
    let page = read("language.html")?;
    let page_en = read("en/language.html")?;
    for lang in &languages {
        let s = &lang.summary;
        let html = render(&page, &[("base", base), ("slug", &s.slug), ("title", &s.endonym)]);
        write(&out.join(&s.slug).join("index.html"), html)?;
        let html = render(&page_en, &[("base", base), ("slug", &s.slug), ("title", &s.endonym)]);
        write(&out.join("en").join(&s.slug).join("index.html"), html)?;
    }
    // Lets GitHub Pages serve files and folders as they are.
    write(&out.join(".nojekyll"), "")?;

    if opts.wasm {
        build_wasm(root, out)?;
    }
    Ok(())
}

fn build_wasm(root: &Path, out: &Path) -> Result<(), String> {
    let status = Command::new("cargo")
        .current_dir(root)
        .args(["build", "-p", "coniugatio-wasm", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if !status.success() {
        return Err("building the WebAssembly module failed. Is the target installed? \
                    Run: rustup target add wasm32-unknown-unknown"
            .into());
    }
    let wasm = root.join("target/wasm32-unknown-unknown/release/coniugatio_wasm.wasm");
    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--no-typescript", "--out-dir"])
        .arg(out.join("pkg"))
        .arg(&wasm)
        .status()
        .map_err(|_| {
            "wasm-bindgen was not found. Install it with: \
             cargo install wasm-bindgen-cli --version 0.2.100 --locked"
                .to_string()
        })?;
    if !status.success() {
        return Err("wasm-bindgen failed".into());
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

/// A small static file server that behaves like GitHub Pages: the site lives
/// under `base`, folders serve their index.html and missing files get 404.html.
fn serve(dir: &Path, base: &str, port: u16) {
    let server = tiny_http::Server::http(("127.0.0.1", port)).expect("could not bind port");
    println!("Mati Mati coniugatio → http://127.0.0.1:{port}{base}");
    for request in server.incoming_requests() {
        let url = request.url().split(['?', '#']).next().unwrap_or("/").to_string();
        if url == "/" && base != "/" {
            let header = tiny_http::Header::from_bytes("Location", base.as_bytes()).unwrap();
            let _ = request.respond(tiny_http::Response::empty(302).with_header(header));
            continue;
        }
        let inside = url.strip_prefix(base.trim_end_matches('/')).filter(|r| r.is_empty() || r.starts_with('/'));
        let safe = inside.is_some() && !url.split('/').any(|seg| seg == "..");
        let mut path = dir.join(inside.unwrap_or("").trim_start_matches('/'));
        if safe && path.is_dir() && !url.ends_with('/') {
            let to = format!("{url}/");
            let header = tiny_http::Header::from_bytes("Location", to.as_bytes()).unwrap();
            let _ = request.respond(tiny_http::Response::empty(301).with_header(header));
            continue;
        }
        if path.is_dir() {
            path = path.join("index.html");
        }
        let (status, file) = if safe && path.is_file() { (200, path) } else { (404, dir.join("404.html")) };
        let body = fs::read(&file).unwrap_or_default();
        let header = tiny_http::Header::from_bytes("Content-Type", content_type(&file)).unwrap();
        let _ = request.respond(tiny_http::Response::from_data(body).with_status_code(status).with_header(header));
    }
}
