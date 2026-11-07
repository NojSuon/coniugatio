// Shared pieces for every Mati Mati page: logo, header, footer and helpers.

// Simple MA/TI monogram, redrawn from the manual (§3). Uses currentColor so it
// works as "positivo" (Tinta) and "negativo" (blanco).
// TODO: replace with the official vector logo once available.
export const LOGO_SVG = `
<svg viewBox="0 0 100 100" role="img" aria-label="Mati Mati" fill="currentColor">
  <path d="M0 46V3h12l11 15 11-15h12v43H35V22L23 36 11 22v24z"/>
  <path fill-rule="evenodd" d="M54 46V14C54 7 59 3 66 3h22c7 0 12 4 12 11v32H89V33H65v13zm11-23h24v-8c0-1-1-2-2-2H67c-1 0-2 1-2 2z"/>
  <path d="M0 54h46v11H28.5v35h-11V65H0z"/>
  <path d="M54 54h46v11H82.5v24H100v11H54V89h17.5V65H54z"/>
</svg>`;

// ---------- Language of the page ----------
// Spanish pages live at /coniugatio/…, English ones at /coniugatio/en/….
export const EN = document.documentElement.lang === "en";
// The Spanish or the English text, for the page's language.
export const t = (es, en) => (EN ? en : es);
// Home page of the current language, relative to <base href>.
export const HOME = EN ? "en/" : "./";

// The same page in the other language: /otomi/ ⇄ /en/otomi/.
function otherLanguageHref() {
  const base = new URL(document.baseURI).pathname;
  const here = location.pathname.startsWith(base) ? location.pathname.slice(base.length) : "";
  const path = EN ? here.replace(/^en(\/|$)/, "") : `en/${here}`;
  return (path || "./") + location.search + location.hash;
}

// Pages use <base href> so the site works under /coniugatio/ on GitHub Pages.
// With a base, "#section" links would point at the home page, so in-page
// links are handled here instead.
document.addEventListener("click", (ev) => {
  const a = ev.target.closest?.("a[href^='#']");
  if (!a) return;
  const target = document.getElementById(a.getAttribute("href").slice(1));
  if (!target) return;
  ev.preventDefault();
  history.pushState(null, "", location.pathname + location.search + a.getAttribute("href"));
  target.scrollIntoView();
  if (target.tabIndex < 0 && !target.matches("a, button, input")) target.setAttribute("tabindex", "-1");
  target.focus({ preventScroll: true });
});

export function escapeHtml(text) {
  return String(text)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

// A word in an indigenous language, in Fredoka Bold (manual §12).
// Fredoka has no glyph for the line-below mark (U+0331, as in u̱), so the
// line is drawn with CSS under the letter and the real mark is kept in a
// visually hidden span: the word looks consistent and still copies as "u̱".
// ISO 639-3 code of the language on this page (ote, itz…), for lang="".
let langCode = "";
export function setLangCode(code) {
  langCode = code || "";
}

export function langWord(text, tag = "span", extra = "") {
  const html = escapeHtml(String(text).normalize("NFC")).replace(
    /(\p{L})̱/gu,
    '<span class="lb">$1<span class="lb-mark">̱</span></span>'
  );
  const cls = `lang${extra ? " " + extra : ""}`;
  const code = langCode ? ` lang="${langCode}"` : "";
  return `<${tag} class="${cls}"${code}>${html}</${tag}>`;
}

export function renderHeader(current) {
  const el = document.querySelector("[data-site-header]");
  if (!el) return;
  const link = (href, label, key) =>
    `<li><a href="${href}"${current === key ? ' aria-current="page"' : ""}>${label}</a></li>`;
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="${HOME}">${LOGO_SVG}<span>Mati Mati</span></a>
      <div class="header-end">
        <nav class="site-nav" aria-label="${t("Principal", "Main")}">
          <ul>
            ${link(HOME, t("Inicio", "Home"), "home")}
            ${link(`${HOME}#lenguas`, t("Lenguas", "Languages"), "lenguas")}
            ${link(`${HOME}#mati`, "Mati Mati", "mati")}
          </ul>
        </nav>
        ${EN
          ? `<a class="lang-switch" href="${otherLanguageHref()}" hreflang="es" lang="es" aria-label="Ver esta página en español">ES</a>`
          : `<a class="lang-switch" href="${otherLanguageHref()}" hreflang="en" lang="en" aria-label="Read this page in English">EN</a>`}
      </div>
    </div>`;
}

export function renderFooter() {
  const el = document.querySelector("[data-site-footer]");
  if (!el) return;
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="${HOME}">${LOGO_SVG}<span>Mati Mati</span></a>
      <p>${t("Aprende, preserva y comparte lenguas indígenas.", "Learn, preserve and share Indigenous languages.")}<br>
        ${t("Síguenos en", "Follow us at")} <a href="https://www.instagram.com/matidemati" rel="noopener">@matidemati</a>.</p>
    </div>`;
}

// Keeps the language switch pointing at the same verb and section after the
// URL changes (?verbo=…, #conjugador).
export function refreshLanguageSwitch() {
  const a = document.querySelector(".lang-switch");
  if (a) a.setAttribute("href", otherLanguageHref());
}
window.addEventListener("hashchange", refreshLanguageSwitch);
