import { renderHeader, renderFooter, langWord as w, escapeHtml as e, setLangCode, t, EN, refreshLanguageSwitch } from "./common.js";

renderHeader("lenguas");
renderFooter();

const root = document.querySelector("[data-language-root]");
const slug = root.dataset.slug;

const res = await fetch(`data/${EN ? "en/" : ""}${encodeURIComponent(slug)}.json`);
if (!res.ok) {
  root.innerHTML = `<div class="section"><div class="wrap"><p class="error">${t("No encontramos esta lengua.", "We couldn't find this language.")}</p></div></div>`;
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
      <h1>${e(lang.variant)}${lang.status === "available" ? "" : t(" · próximamente", " · coming soon")}</h1>
    </div>
  </section>`;
}

function subnav() {
  const items = [
    ["conjugador", t("Conjugador", "Conjugator")],
    ["tu-verbo", t("Conjuga tu verbo", "Conjugate your verb")],
    ["lengua", t("Conoce la lengua", "About the language")],
    ["ortografia", t("Ortografía", "Alphabet")],
    ["gramatica", t("Cómo se conjuga", "How verbs work")],
  ];
  return `<nav class="subnav" aria-label="${t("Secciones", "Sections")}"><div class="wrap"><ul>
    ${items.map(([id, label]) => `<li><a href="#${id}">${label}</a></li>`).join("")}
  </ul></div></nav>`;
}

function about() {
  return `
  <section class="section" id="lengua" aria-labelledby="lengua-t">
    <div class="wrap stack">
      <h2 id="lengua-t">${t("Conoce la lengua", "About the language")}</h2>
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
      <h2 id="orto-t">${t("Ortografía", "Alphabet")}</h2>
      <p class="lead">${e(o.intro)}</p>
      <ul class="letters">
        ${o.letters
          .map(
            (l) => `<li><span class="letter-head">${w(l.letter, "span", "letter")}${
              l.ipa ? `<span class="ipa" title="${t("Pronunciación (AFI)", "Pronunciation (IPA)")}">[${e(l.ipa)}]</span>` : ""
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
      <h2 id="gram-t">${t("Cómo se conjuga", "How verbs work")}</h2>
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
      <h2 id="conj-t">${t("Conjugador", "Conjugator")}</h2>
      <p class="lead">${e(ui.hint || t("Escribe la raíz de un verbo, como la encuentras en el diccionario.", "Type the root of a verb, as you find it in the dictionary."))}</p>
      <form class="conjugator-form" data-form>
        <div class="field">
          <label for="verb">${e(ui.label || t("Raíz del verbo", "Verb root"))}</label>
          <input id="verb" name="verb" autocomplete="off" autocapitalize="off" spellcheck="false"
                 placeholder="${e(ui.placeholder || "")}" required lang="${e(lang.lang_code || "")}">
          ${charpad("verb")}
        </div>
        <button class="button" type="submit">${t("Conjugar", "Conjugate")}</button>
      </form>
      <ul class="samples" aria-label="${t("Verbos de ejemplo", "Example verbs")}">
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
  return `<div class="charpad" aria-label="${t("Letras especiales", "Special letters")}">
    ${keys.map((k) => `<button type="button" data-char="${e(k)}" data-target="${target}" aria-label="${t("Agregar", "Add")} ${e(k)}">${e(k)}</button>`).join("")}
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
      <h2 id="own-t">${t("Conjuga tu verbo", "Conjugate your verb")}</h2>
      <p class="lead">${e(own.hint || t("Escribe cualquier verbo y su significado en español. Lo conjugamos con las reglas de la lengua.", "Type any verb and its meaning in English. We conjugate it with the rules of the language."))}</p>
      <form class="conjugator-form own-form" data-own-form>
        <div class="field">
          <label for="own-verb">${e(own.label || ui.label || t("Verbo", "Verb"))}</label>
          <input id="own-verb" name="verb" autocomplete="off" autocapitalize="off" spellcheck="false"
                 placeholder="${e(own.placeholder || "")}" required lang="${e(lang.lang_code || "")}">
          ${charpad("own-verb")}
        </div>
        <div class="field">
          <label for="own-es">${t("Significado en español", "Meaning in English")}</label>
          <input id="own-es" name="es" autocomplete="off" placeholder="${e(own.es_placeholder || t("trabajar", "work"))}" lang="${t("es", "en")}">
          <p class="caption field-help">${t("En infinitivo: termina en -ar, -er o -ir.", "The plain verb: «sleep», «do it», «drink water».")}</p>
        </div>
        ${types.length ? `<div class="field">
          <label for="own-kind">${t("Tipo de verbo", "Verb type")}</label>
          <select id="own-kind" name="kind">
            ${types.map((t) => `<option value="${e(t.id)}">${e(t.label)}</option>`).join("")}
          </select>
          ${own.types_help ? `<p class="caption field-help">${e(own.types_help)}</p>` : ""}
        </div>` : ""}
        <button class="button" type="submit">${t("Conjugar mi verbo", "Conjugate my verb")}</button>
      </form>
      <p class="error" data-own-error hidden></p>
      <div data-own-result aria-live="polite"></div>
      <div class="card card--blanca learn-more">
        <h3>${t("¿Quieres saber más?", "Want to know more?")}</h3>
        <ul>
          <li><a href="#lengua">${t("Conoce la lengua", "About the language")}</a>: ${t("quién la habla y dónde.", "who speaks it and where.")}</li>
          <li><a href="#ortografia">${t("Ortografía", "Alphabet")}</a>: ${t("cada letra con una palabra de ejemplo.", "every letter with an example word.")}</li>
          <li><a href="#gramatica">${t("Cómo se conjuga", "How verbs work")}</a>: ${t("las piezas de cada verbo y sus reglas.", "the pieces of each verb and their rules.")}</li>
        </ul>
      </div>
    </div>
  </section>`;
}

function source() {
  if (!lang.source) return "";
  return `<div class="wrap"><p class="source"><strong>${t("Fuente", "Source")}:</strong> ${e(lang.source.full)}</p></div>`;
}

function comingSoon() {
  return `<section class="section"><div class="wrap stack">
    <h2>${t("Estamos preparando esta lengua", "We are preparing this language")}</h2>
    <p>${e(lang.summary)}</p>
    <p>${t(`¿Hablas ${e(lang.name)} o la estás aprendiendo? Cuéntanos en`, `Do you speak ${e(lang.name)} or are you learning it? Tell us at`)} <a href="https://www.instagram.com/matidemati">@matidemati</a>.</p>
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
  const hasGloss = data.tables.some((tb) => tb.rows.some((r) => r.es_singular));
  const status = own && !data.known
    ? `<p class="notice result-note">${t("Conjugamos tu verbo con las reglas generales. Algunos verbos tienen formas propias: revisa el diccionario.", "We conjugated your verb with the general rules. Some verbs have forms of their own: check the dictionary.")}${
        data.meaning && !hasGloss
          ? t(` No pudimos conjugar «${e(data.meaning)}» en español: escríbelo en infinitivo, como «trabajar».`, ` We couldn't conjugate «${e(data.meaning)}» in English: write a plain verb, like «work».`)
          : ""
      }</p>`
    : data.known
    ? `<p class="notice notice--ok result-note">${w(data.root)} «${e(data.meaning)}» ${t("está en el diccionario", "is in the dictionary")}${
        data.note ? ` (${e(data.note)}).` : "."
      }</p>`
    : `<p class="notice result-note">${t("Este verbo no está en nuestra lista, así que aplicamos las reglas generales. Revisa sus formas en el diccionario: algunos verbos cambian de manera distinta.", "This verb isn't in our list, so we applied the general rules. Check its forms in the dictionary: some verbs change in their own way.")}</p>`;

  const es = (text) => (text ? `<span class="es" lang="${t("es", "en")}">${e(text)}</span>` : "");
  // A dash where a construction has no such person (imperatives are 2nd
  // person only); Otomí explains its missing 3rd person plural instead.
  const EMPTY = `<span class="caption" aria-label="${t("no existe", "does not exist")}">—</span>`;
  const noPlural = (lang.conjugator || {}).no_plural
    ? `<span class="caption">${e(lang.conjugator.no_plural)}</span>`
    : EMPTY;

  const table = (tb) => `
    <div class="tense">
      <h4>${e(tb.name)}</h4>
      <p class="caption">${e(tb.description)}</p>
      <div class="table-scroll"><table class="data">
        <thead><tr><th scope="col">${t("Persona", "Person")}</th><th scope="col">Singular</th><th scope="col">Plural</th></tr></thead>
        <tbody>${tb.rows
          .map(
            (r) => `<tr>
              <th scope="row" class="person">${e(r.person)}</th>
              <td>${r.singular ? w(r.singular) : EMPTY}${es(r.es_singular)}</td>
              <td>${r.plural ? w(r.plural) : noPlural}${es(r.es_plural)}</td>
            </tr>`
          )
          .join("")}</tbody>
      </table></div>
    </div>`;

  // One group of tables at a time, with buttons to switch between them.
  const shown = groups.includes(lastGroup) ? lastGroup : groups[0];
  const tabs =
    groups.length < 2
      ? ""
      : `<div class="group-tabs" role="tablist" aria-label="${t("Grupos de tiempos", "Groups of tenses")}">${groups
          .map((g, i) => {
            const n = data.tables.filter((t) => t.group === g).length;
            return `<button type="button" role="tab" data-group-tab="${i}" aria-selected="${g === shown}">${e(g)} <span class="count">${n}</span></button>`;
          })
          .join("")}</div>`;
  const panels = groups
    .map((g, i) => {
      const tables = data.tables.filter((t) => t.group === g).map(table).join("");
      const hidden = g === shown ? "" : " hidden";
      return `<div class="tense-group" role="tabpanel" data-group-panel="${i}" data-group-name="${e(g)}"${hidden}>
          <h3>${e(g)}</h3><div class="tense-grid">${tables}</div></div>`;
    })
    .join("");

  return `
    <div class="result-head">${w(data.root)}${data.meaning ? `<span class="caption">«${e(data.meaning)}»</span>` : ""}</div>
    ${status}
    ${tabs}
    ${panels}`;
}

// The group shown last stays selected when another verb is conjugated.
let lastGroup = null;

document.addEventListener("click", (ev) => {
  const tab = ev.target.closest?.("[data-group-tab]");
  if (!tab) return;
  const box = tab.closest("[data-result], [data-own-result]");
  box.querySelectorAll("[data-group-tab]").forEach((b) => b.setAttribute("aria-selected", String(b === tab)));
  box.querySelectorAll("[data-group-panel]").forEach((p) => {
    p.hidden = p.dataset.groupPanel !== tab.dataset.groupTab;
    if (!p.hidden) lastGroup = p.dataset.groupName;
  });
});

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
  const pageLang = t("es", "en");
  // The verb in the address, so a conjugation can be shared: ?verbo= / ?verb=.
  const param = t("verbo", "verb");

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
    const data = await show((m) => m.conjugate(conjugationJson, verb, pageLang), out, err, false);
    if (data) {
      history.replaceState(null, "", `${location.pathname}?${param}=${encodeURIComponent(data.root)}#conjugador`);
      refreshLanguageSwitch();
    }
  }

  const ownForm = document.querySelector("[data-own-form]");
  ownForm.addEventListener("submit", (ev) => {
    ev.preventDefault();
    const verb = ownForm.querySelector("#own-verb").value;
    const es = ownForm.querySelector("#own-es").value;
    const kind = ownForm.querySelector("#own-kind")?.value ?? "";
    show(
      (m) => m.conjugate_with(conjugationJson, verb, es, kind, pageLang),
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

  const query = new URLSearchParams(location.search);
  const initial = query.get("verbo") ?? query.get("verb");
  if (initial) {
    input.value = initial;
    run(initial);
  }
}
