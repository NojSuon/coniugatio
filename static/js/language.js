import { renderHeader, renderFooter, langWord as w, escapeHtml as e, setLangCode } from "./common.js";

renderHeader("lenguas");
renderFooter();

const root = document.querySelector("[data-language-root]");
const slug = root.dataset.slug;

const res = await fetch(`data/${encodeURIComponent(slug)}.json`);
if (!res.ok) {
  root.innerHTML = `<div class="section"><div class="wrap"><p class="error">No encontramos esta lengua.</p></div></div>`;
  throw new Error("language not found");
}
const lang = await res.json();

document.title = `${lang.endonym} · Mati Mati`;
document.body.dataset.serie = lang.series;
setLangCode(lang.lang_code);

// ---------- Sections ----------

function hero() {
  return `
  <section class="lang-hero">
    <div class="wrap">
      ${w(lang.endonym)}
      <h1>${e(lang.variant)}${lang.status === "available" ? "" : " · próximamente"}</h1>
    </div>
  </section>`;
}

function subnav() {
  const items = [
    ["conjugador", "Conjugador"],
    ["tu-verbo", "Conjuga tu verbo"],
    ["lengua", "Conoce la lengua"],
    ["ortografia", "Ortografía"],
    ["gramatica", "Cómo se conjuga"],
  ];
  return `<nav class="subnav" aria-label="Secciones"><div class="wrap"><ul>
    ${items.map(([id, label]) => `<li><a href="#${id}">${label}</a></li>`).join("")}
  </ul></div></nav>`;
}

function about() {
  return `
  <section class="section" id="lengua" aria-labelledby="lengua-t">
    <div class="wrap stack">
      <h2 id="lengua-t">Conoce la lengua</h2>
      <p><span class="tag">${e(lang.region)}</span></p>
      ${lang.about.map((p) => `<p>${e(p)}</p>`).join("")}
    </div>
  </section>`;
}

function orthography() {
  const o = lang.orthography;
  const vowelList = (list) =>
    `<ul class="vowels">${list.map((v) => `<li>${w(v.letter)}<span class="ipa">[${e(v.ipa)}]</span></li>`).join("")}</ul>`;
  return `
  <section class="section" id="ortografia" aria-labelledby="orto-t">
    <div class="wrap">
      <h2 id="orto-t">Ortografía</h2>
      <p class="lead">${e(o.intro)}</p>
      <ul class="letters">
        ${o.letters
          .map(
            (l) => `<li><span class="letter-head">${w(l.letter, "span", "letter")}${
              l.ipa ? `<span class="ipa" title="Pronunciación (AFI)">[${e(l.ipa)}]</span>` : ""
            }</span>
              ${w(l.word, "span", "word")}
              <span class="meaning">${e(l.meaning)}</span></li>`
          )
          .join("")}
      </ul>
      <div class="grid-2" style="margin-top:24px">
        ${o.vowels.map((g) => `<div class="card"><h3>${e(g.title)}</h3>${vowelList(g.items)}</div>`).join("")}
      </div>
      <div class="card card--blanca stack" style="margin-top:24px">
        ${o.notes.map((n) => `<p>${e(n)}</p>`).join("")}
      </div>
    </div>
  </section>`;
}

// The grammar section is a list of blocks, so each language can describe
// its own system (see "grammar.blocks" in data/languages/*.json).
const change = (x) =>
  `<li>${w(x.from)}<span class="arrow">→</span>${w(x.to)}<span class="caption">«${e(x.meaning)}»</span></li>`;

const heading = (b) =>
  `${b.title ? `<h3 class="block-title">${e(b.title)}</h3>` : ""}${
    b.description ? `<p class="block-desc">${e(b.description)}</p>` : ""
  }`;

const BLOCKS = {
  formula: (b) => `${heading(b)}<ol class="formula">
      ${b.items.map((f) => `<li><span class="slot">${e(f.slot)}</span>${w(f.example)}<p>${e(f.description)}</p></li>`).join("")}
    </ol>`,

  pairs: (b) => `${heading(b)}<ul class="pairs">
      ${b.items.map((p) => `<li>${w(p.text)}<span>«${e(p.translation)}»</span></li>`).join("")}
    </ul>`,

  cards: (b) => `${heading(b)}<div class="grid-3">
      ${b.items
        .map(
          (r) => `<div class="card">
            ${r.tag ? `<span class="tag tag--suave">${e(r.tag)}</span>` : ""}
            <h3 style="margin-top:10px">${e(r.name)}</h3>
            <p style="font-size:16px;margin-top:6px">${e(r.description)}</p>
            ${r.examples ? `<ul class="change-list">${r.examples.map(change).join("")}</ul>` : ""}
          </div>`
        )
        .join("")}
    </div>`,

  changes: (b) => `<div class="card card--blanca">${heading(b)}
      <ul class="change-list">${b.examples.map(change).join("")}</ul></div>`,

  table: (b) => `<div class="card card--blanca">${heading(b)}
      <div class="table-scroll"><table class="data">
        <thead><tr>${b.columns.map((c) => `<th scope="col">${e(c)}</th>`).join("")}</tr></thead>
        <tbody>${b.rows
          .map((r) => `<tr><th scope="row">${e(r.label)}</th>${r.cells.map((c) => `<td>${w(c)}</td>`).join("")}</tr>`)
          .join("")}</tbody>
      </table></div>
      ${b.note ? `<p class="caption" style="margin-top:12px">${e(b.note)}</p>` : ""}
      ${b.example ? `<p style="margin-top:12px">${w(b.example.text)} «${e(b.example.translation)}»</p>` : ""}
    </div>`,

  grid: (b) => `<div class="grid-2">${b.blocks.map(block).join("")}</div>`,
};

function block(b) {
  const render = BLOCKS[b.type];
  return render ? `<div class="block">${render(b)}</div>` : "";
}

function grammar() {
  const g = lang.grammar;
  return `
  <section class="section" id="gramatica" aria-labelledby="gram-t">
    <div class="wrap">
      <h2 id="gram-t">Cómo se conjuga</h2>
      <p class="lead">${e(g.intro)}</p>
      ${g.blocks.map(block).join("")}
    </div>
  </section>`;
}


function conjugator() {
  const ui = lang.conjugator || {};
  const samples = lang.conjugation.lexicon;
  const word = (s) => s.entry ?? s.root;
  return `
  <section class="section" id="conjugador" aria-labelledby="conj-t">
    <div class="wrap">
      <h2 id="conj-t">Conjugador</h2>
      <p class="lead">${e(ui.hint || "Escribe la raíz de un verbo, como la encuentras en el diccionario.")}</p>
      <form class="conjugator-form" data-form>
        <div class="field">
          <label for="verb">${e(ui.label || "Raíz del verbo")}</label>
          <input id="verb" name="verb" autocomplete="off" autocapitalize="off" spellcheck="false"
                 placeholder="${e(ui.placeholder || "")}" required lang="${e(lang.lang_code || "")}">
          ${charpad("verb")}
        </div>
        <button class="button" type="submit">Conjugar</button>
      </form>
      <ul class="samples" aria-label="Verbos de ejemplo">
        ${samples
          .map((s) => `<li><button type="button" data-sample="${e(word(s))}">${w(word(s))} · ${e(s.meaning)}</button></li>`)
          .join("")}
      </ul>
      <p class="error" data-error hidden></p>
      <div data-result aria-live="polite"></div>
    </div>
  </section>`;
}

function charpad(target) {
  const keys = (lang.conjugator || {}).keyboard || [];
  if (!keys.length) return "";
  return `<div class="charpad" aria-label="Letras especiales">
    ${keys.map((k) => `<button type="button" data-char="${e(k)}" data-target="${target}" aria-label="Agregar ${e(k)}">${e(k)}</button>`).join("")}
  </div>`;
}

// "Conjuga tu verbo": any verb, with its Spanish meaning and (optionally)
// its type, conjugated with the language's rules.
function ownVerb() {
  const ui = lang.conjugator || {};
  const own = ui.own || {};
  const types = ui.types || [];
  return `
  <section class="section" id="tu-verbo" aria-labelledby="own-t">
    <div class="wrap">
      <h2 id="own-t">Conjuga tu verbo</h2>
      <p class="lead">${e(own.hint || "Escribe cualquier verbo y su significado en español. Lo conjugamos con las reglas de la lengua.")}</p>
      <form class="conjugator-form own-form" data-own-form>
        <div class="field">
          <label for="own-verb">${e(own.label || ui.label || "Verbo")}</label>
          <input id="own-verb" name="verb" autocomplete="off" autocapitalize="off" spellcheck="false"
                 placeholder="${e(own.placeholder || "")}" required lang="${e(lang.lang_code || "")}">
          ${charpad("own-verb")}
        </div>
        <div class="field">
          <label for="own-es">Significado en español</label>
          <input id="own-es" name="es" autocomplete="off" placeholder="${e(own.es_placeholder || "trabajar")}" lang="es">
          <p class="caption field-help">En infinitivo: termina en -ar, -er o -ir.</p>
        </div>
        ${types.length ? `<div class="field">
          <label for="own-kind">Tipo de verbo</label>
          <select id="own-kind" name="kind">
            ${types.map((t) => `<option value="${e(t.id)}">${e(t.label)}</option>`).join("")}
          </select>
          ${own.types_help ? `<p class="caption field-help">${e(own.types_help)}</p>` : ""}
        </div>` : ""}
        <button class="button" type="submit">Conjugar mi verbo</button>
      </form>
      <p class="error" data-own-error hidden></p>
      <div data-own-result aria-live="polite"></div>
      <div class="card card--blanca learn-more">
        <h3>¿Quieres saber más?</h3>
        <ul>
          <li><a href="#lengua">Conoce la lengua</a>: quién la habla y dónde.</li>
          <li><a href="#ortografia">Ortografía</a>: cada letra con una palabra de ejemplo.</li>
          <li><a href="#gramatica">Cómo se conjuga</a>: las piezas de cada verbo y sus reglas.</li>
        </ul>
      </div>
    </div>
  </section>`;
}

function source() {
  if (!lang.source) return "";
  return `<div class="wrap"><p class="source"><strong>Fuente:</strong> ${e(lang.source.full)}</p></div>`;
}

function comingSoon() {
  return `<section class="section"><div class="wrap stack">
    <h2>Estamos preparando esta lengua</h2>
    <p>${e(lang.summary)}</p>
    <p>¿Hablas ${e(lang.name)} o la estás aprendiendo? Cuéntanos en <a href="https://www.instagram.com/matidemati">@matidemati</a>.</p>
  </div></section>`;
}

if (lang.status !== "available") {
  root.innerHTML = hero() + comingSoon();
} else {
  root.innerHTML = hero() + subnav() + conjugator() + ownVerb() + about() + orthography() + grammar() + source();
  setupConjugator();
}

// ---------- Conjugator behaviour ----------

function renderResult(data, own = false) {
  const groups = [...new Set(data.tables.map((t) => t.group))];
  const hasSpanish = data.tables.some((t) => t.rows.some((r) => r.es_singular));
  const status = own && !data.known
    ? `<p class="notice result-note">Conjugamos tu verbo con las reglas generales. Algunos verbos tienen formas propias: revisa el diccionario.${
        data.meaning && !hasSpanish
          ? ` No pudimos conjugar «${e(data.meaning)}» en español: escríbelo en infinitivo, como «trabajar».`
          : ""
      }</p>`
    : data.known
    ? `<p class="notice notice--ok result-note">${w(data.root)} «${e(data.meaning)}» está en el diccionario${
        data.note ? ` (${e(data.note)}).` : "."
      }</p>`
    : `<p class="notice result-note">Este verbo no está en nuestra lista, así que aplicamos las reglas generales. Revisa sus formas en el diccionario: algunos verbos cambian de manera distinta.</p>`;

  const es = (text) => (text ? `<span class="es" lang="es">${e(text)}</span>` : "");

  const table = (t) => `
    <div class="tense">
      <h4>${e(t.name)}</h4>
      <p class="caption">${e(t.description)}</p>
      <table class="data">
        <thead><tr><th scope="col">Persona</th><th scope="col">Singular</th><th scope="col">Plural</th></tr></thead>
        <tbody>${t.rows
          .map(
            (r) => `<tr>
              <th scope="row" class="person">${e(r.person)}</th>
              <td>${w(r.singular)}${es(r.es_singular)}</td>
              <td>${
                r.plural ? w(r.plural) : `<span class="caption">igual, con ${w("ya")} en el sustantivo</span>`
              }${es(r.es_plural)}</td>
            </tr>`
          )
          .join("")}</tbody>
      </table>
    </div>`;

  return `
    <div class="result-head">${w(data.root)}${data.meaning ? `<span class="caption">«${e(data.meaning)}»</span>` : ""}</div>
    ${status}
    ${groups
      .map(
        (g) => `<div class="tense-group"><h3>${e(g)}</h3>
          <div class="tense-grid">${data.tables.filter((t) => t.group === g).map(table).join("")}</div></div>`
      )
      .join("")}`;
}

function setupConjugator() {
  const form = document.querySelector("[data-form]");
  const input = form.querySelector("#verb");
  const out = document.querySelector("[data-result]");
  const err = document.querySelector("[data-error]");

  // The conjugator is Rust compiled to WebAssembly (crates/wasm).
  const engine = import("../../pkg/coniugatio_wasm.js").then(async (m) => {
    await m.default();
    return m;
  });
  const conjugationJson = JSON.stringify(lang.conjugation);

  // Runs the engine and shows the tables (or the error) in a section.
  async function show(call, target, errorBox, own) {
    errorBox.hidden = true;
    try {
      const m = await engine;
      let data;
      try {
        data = JSON.parse(call(m));
      } catch (message) {
        throw new Error(String(message)); // wasm-bindgen throws the Rust error string
      }
      target.innerHTML = renderResult(data, own);
      return data;
    } catch (x) {
      errorBox.textContent = x.message;
      errorBox.hidden = false;
      return null;
    }
  }

  async function run(verb) {
    const data = await show((m) => m.conjugate(conjugationJson, verb), out, err, false);
    if (data) history.replaceState(null, "", `${location.pathname}?verbo=${encodeURIComponent(data.root)}#conjugador`);
  }

  const ownForm = document.querySelector("[data-own-form]");
  ownForm.addEventListener("submit", (ev) => {
    ev.preventDefault();
    const verb = ownForm.querySelector("#own-verb").value;
    const es = ownForm.querySelector("#own-es").value;
    const kind = ownForm.querySelector("#own-kind")?.value ?? "";
    show(
      (m) => m.conjugate_with(conjugationJson, verb, es, kind),
      document.querySelector("[data-own-result]"),
      document.querySelector("[data-own-error]"),
      true
    );
  });

  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    run(input.value);
  });

  document.querySelectorAll("[data-sample]").forEach((b) =>
    b.addEventListener("click", () => {
      input.value = b.dataset.sample;
      run(input.value);
    })
  );

  document.querySelectorAll("[data-char]").forEach((b) =>
    b.addEventListener("click", () => {
      const field = document.getElementById(b.dataset.target);
      const { selectionStart: s = field.value.length, selectionEnd: end = field.value.length } = field;
      field.value = field.value.slice(0, s) + b.dataset.char + field.value.slice(end);
      field.focus();
      const pos = s + b.dataset.char.length;
      field.setSelectionRange(pos, pos);
    })
  );

  const initial = new URLSearchParams(location.search).get("verbo");
  if (initial) {
    input.value = initial;
    run(initial);
  }
}
