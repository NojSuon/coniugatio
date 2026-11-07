import { renderHeader, renderFooter, langWord as w, escapeHtml as e } from "./common.js";

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

// ---------- Sections ----------

function hero() {
  return `
  <section class="lang-hero">
    <div class="pattern-p02 pattern-strip" aria-hidden="true"></div>
    <div class="wrap">
      <span class="tag">${lang.status === "available" ? "Conjugador" : "Próximamente"}</span>
      ${w(lang.endonym)}
      <h1>${e(lang.variant)}</h1>
      <p>${e(lang.region)}</p>
    </div>
  </section>`;
}

function subnav() {
  const items = [
    ["conjugador", "Conjugador"],
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
        <div class="card"><h3>Vocales orales</h3>${vowelList(o.vowels.oral)}</div>
        <div class="card"><h3>Vocales nasales</h3>${vowelList(o.vowels.nasal)}</div>
      </div>
      <div class="card card--blanca stack" style="margin-top:24px">
        ${o.notes.map((n) => `<p>${e(n)}</p>`).join("")}
      </div>
    </div>
  </section>`;
}

function grammar() {
  const g = lang.grammar;
  const change = (x) =>
    `<li>${w(x.from)}<span class="arrow">→</span>${w(x.to)}<span class="caption">«${e(x.meaning)}»</span></li>`;
  return `
  <section class="section" id="gramatica" aria-labelledby="gram-t">
    <div class="wrap">
      <h2 id="gram-t">Cómo se conjuga</h2>
      <p class="lead">${e(g.intro)}</p>
      <ol class="formula">
        ${g.formula
          .map((f) => `<li><span class="slot">${e(f.slot)}</span>${w(f.example)}<p>${e(f.description)}</p></li>`)
          .join("")}
      </ol>

      <h3 style="margin:36px 0 8px">El tono del proclítico cambia el significado</h3>
      <p style="margin-bottom:14px">Fíjate en la tilde: marca un tono alto.</p>
      <ul class="pairs">
        ${g.tone_pairs.map((p) => `<li>${w(p.text)}<span>«${e(p.translation)}»</span></li>`).join("")}
      </ul>

      <h3 style="margin:36px 0 8px">La raíz también cambia</h3>
      <p style="margin-bottom:14px">${e(g.root_changes.intro)}</p>
      <div class="grid-3">
        ${g.root_changes.rules
          .map(
            (r) => `<div class="card">
              <span class="tag tag--suave">${e(r.where)}</span>
              <h3 style="margin-top:10px">${e(r.name)}</h3>
              <p style="font-size:16px;margin-top:6px">${e(r.description)}</p>
              <ul class="change-list">${r.examples.map(change).join("")}</ul>
            </div>`
          )
          .join("")}
      </div>

      <div class="grid-2" style="margin-top:24px">
        <div class="card card--blanca">
          <h3>El plural con -hu̱</h3>
          <p style="font-size:16px;margin-top:6px">${e(g.plural.description)}</p>
          <ul class="change-list">${g.plural.examples.map(change).join("")}</ul>
        </div>
        <div class="card card--blanca">
          <h3>Ser algo: proclítico + ra</h3>
          <p style="font-size:16px;margin:6px 0 12px">${e(g.predicate.description)}</p>
          <div class="table-scroll"><table class="data">
            <thead><tr><th scope="col">Tiempo</th><th scope="col">1.ª</th><th scope="col">2.ª</th><th scope="col">3.ª</th></tr></thead>
            <tbody>${g.predicate.tenses
              .map((t) => `<tr><th scope="row">${e(t.name)}</th>${t.forms.map((f) => `<td>${w(f)}</td>`).join("")}</tr>`)
              .join("")}</tbody>
          </table></div>
          <p style="margin-top:12px">${w(g.predicate.example.text)} «${e(g.predicate.example.translation)}»</p>
        </div>
      </div>

      <h3 style="margin:36px 0 12px">Frases de ejemplo</h3>
      <ul class="pairs">
        ${g.examples.map((x) => `<li>${w(x.text)}<span>«${e(x.translation)}»</span></li>`).join("")}
      </ul>
    </div>
  </section>`;
}

const SPECIAL = ["'", "ä", "e̱", "o̱", "u̱"];

function conjugator() {
  const samples = lang.conjugation.lexicon;
  return `
  <section class="section" id="conjugador" aria-labelledby="conj-t">
    <div class="wrap">
      <h2 id="conj-t">Conjugador</h2>
      <p class="lead">Escribe la raíz de un verbo, como la encuentras en el diccionario.</p>
      <form class="conjugator-form" data-form>
        <div class="field">
          <label for="verb">Raíz del verbo</label>
          <input id="verb" name="verb" autocomplete="off" autocapitalize="off" spellcheck="false"
                 placeholder="pädi" required lang="ote">
          <div class="charpad" aria-label="Letras especiales">
            ${SPECIAL.map((c) => `<button type="button" data-char="${c}" aria-label="Agregar ${c}">${c}</button>`).join("")}
          </div>
        </div>
        <button class="button" type="submit">Conjugar</button>
      </form>
      <ul class="samples" aria-label="Verbos de ejemplo">
        ${samples
          .map((s) => `<li><button type="button" data-sample="${e(s.root)}">${w(s.root)} · ${e(s.meaning)}</button></li>`)
          .join("")}
      </ul>
      <p class="error" data-error hidden></p>
      <div data-result aria-live="polite"></div>
      <div class="card card--blanca learn-more">
        <h3>¿Quieres saber más?</h3>
        <ul>
          <li><a href="#lengua">Conoce la lengua</a>: quién la habla y dónde.</li>
          <li><a href="#ortografia">Ortografía</a>: cada letra con una palabra de ejemplo.</li>
          <li><a href="#gramatica">Cómo se conjuga</a>: los proclíticos y por qué cambia la raíz.</li>
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
  root.innerHTML = hero() + subnav() + conjugator() + about() + orthography() + grammar() + source();
  setupConjugator();
}

// ---------- Conjugator behaviour ----------

function renderResult(data) {
  const groups = [...new Set(data.tables.map((t) => t.group))];
  const status = data.known
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

  async function run(verb) {
    err.hidden = true;
    try {
      const { conjugate } = await engine;
      let data;
      try {
        data = JSON.parse(conjugate(conjugationJson, verb));
      } catch (message) {
        throw new Error(String(message)); // wasm-bindgen throws the Rust error string
      }
      out.innerHTML = renderResult(data);
      history.replaceState(null, "", `${location.pathname}?verbo=${encodeURIComponent(data.root)}#conjugador`);
    } catch (x) {
      err.textContent = x.message;
      err.hidden = false;
    }
  }

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
      const { selectionStart: s = input.value.length, selectionEnd: end = input.value.length } = input;
      input.value = input.value.slice(0, s) + b.dataset.char + input.value.slice(end);
      input.focus();
      const pos = s + b.dataset.char.length;
      input.setSelectionRange(pos, pos);
    })
  );

  const initial = new URLSearchParams(location.search).get("verbo");
  if (initial) {
    input.value = initial;
    run(initial);
  }
}
