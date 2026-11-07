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

// A word in an indigenous language. Fredoka cannot compose the line-below
// mark (U+0331), so those words switch to Nunito Sans Bold (manual §5).
export function langWord(text, tag = "span", extra = "") {
  const compose = /̱/.test(String(text));
  const cls = `lang${compose ? " lang--compose" : ""}${extra ? " " + extra : ""}`;
  return `<${tag} class="${cls}" lang="ote">${escapeHtml(text)}</${tag}>`;
}

export function renderHeader(current) {
  const el = document.querySelector("[data-site-header]");
  if (!el) return;
  const link = (href, label, key) =>
    `<li><a href="${href}"${current === key ? ' aria-current="page"' : ""}>${label}</a></li>`;
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="./">${LOGO_SVG}<span>Mati Mati</span></a>
      <nav class="site-nav" aria-label="Principal">
        <ul>
          ${link("./", "Inicio", "home")}
          ${link("./#lenguas", "Lenguas", "lenguas")}
          ${link("./#mati", "Mati Mati", "mati")}
        </ul>
      </nav>
    </div>`;
}

export function renderFooter() {
  const el = document.querySelector("[data-site-footer]");
  if (!el) return;
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="./">${LOGO_SVG}<span>Mati Mati</span></a>
      <p>Aprende, preserva y comparte lenguas indígenas.<br>
        Síguenos en <a href="https://www.instagram.com/matidemati" rel="noopener">@matidemati</a>.</p>
    </div>`;
}
