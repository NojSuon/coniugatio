// Shared pieces for every Mati Mati page: logo, header, footer and helpers.

// The Mati Mati logo, as in chinamitl: Petróleo on light pages, white on
// dark ones (and always white on the Berenjena footer).
const LOGO = (size = 44, alt = "Mati Mati") => `
  <img class="logo logo-light" src="static/img/mati-logo-petroleo.png" alt="${alt}" width="${size}" height="${size}">
  <img class="logo logo-dark" src="static/img/mati-logo-blanco.png" alt="${alt}" width="${size}" height="${size}">`;
const LOGO_WHITE = (size = 44) =>
  `<img class="logo" src="static/img/mati-logo-blanco.png" alt="Mati Mati" width="${size}" height="${size}">`;
const EMAIL = "matimati.info@gmail.com";

// ---------- Language of the page ----------
// Spanish pages live at /coniugatio/…, English ones at /coniugatio/en/…,
// Chinese ones at /coniugatio/zh/….
const docLang = document.documentElement.lang.split("-")[0]; // zh-Hans → zh
export const LANG = ["en", "zh"].includes(docLang) ? docLang : "es";
export const EN = LANG === "en";
// The Spanish, English or Chinese text, for the page's language.
export const t = (es, en, zh) => (LANG === "zh" ? zh ?? en : EN ? en : es);
// Folder of the page's language, relative to <base href>: "", "en/", "zh/".
export const PREFIX = LANG === "es" ? "" : `${LANG}/`;
// Home page of the current language, relative to <base href>.
export const HOME = PREFIX || "./";

// Donations, on Ko-fi.
export const DONATE_URL = "https://ko-fi.com/suondilut";
const donateLink = (cls) =>
  `<a class="${cls}" href="${DONATE_URL}" target="_blank" rel="noopener"><span aria-hidden="true">♥</span><span class="lbl">${t("Dona", "Donate", "捐助")}</span><span class="visually-hidden"> ${t("(en Ko-fi, se abre en otra pestaña)", "(on Ko-fi, opens in a new tab)", "（在 Ko-fi 上，在新标签页打开）")}</span></a>`;

// The same page in another language: /otomi/ ⇄ /en/otomi/ ⇄ /zh/otomi/.
// The verb travels too, under the other page's parameter (?verbo=, ?verb=).
const SWITCH = [
  { lang: "es", label: "ES", name: "Ver esta página en español" },
  { lang: "en", label: "EN", name: "Read this page in English" },
  { lang: "zh", label: "中文", name: "用中文阅读此页" },
];
function languageHref(target) {
  const base = new URL(document.baseURI).pathname;
  const here = location.pathname.startsWith(base) ? location.pathname.slice(base.length) : "";
  const path = (target === "es" ? "" : `${target}/`) + here.replace(/^(en|zh)(\/|$)/, "");
  const query = new URLSearchParams(location.search);
  const verb = query.get("verbo") ?? query.get("verb");
  const search = verb ? `?${target === "es" ? "verbo" : "verb"}=${encodeURIComponent(verb)}` : "";
  return (path || "./") + search + location.hash;
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

// ---------- Light / dark theme ----------
// Light by default. The <head> of each page sets <html data-theme> before
// the first paint; the header button switches it and remembers the choice.
const MOON_SVG = `<svg class="icon-moon" viewBox="0 0 24 24" aria-hidden="true" fill="currentColor"><path d="M20.5 14.2A8.5 8.5 0 0 1 9.8 3.5a8.5 8.5 0 1 0 10.7 10.7z"/></svg>`;
const SUN_SVG = `<svg class="icon-sun" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><circle cx="12" cy="12" r="4.2" fill="currentColor" stroke="none"/><path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6"/></svg>`;
const isDark = () => document.documentElement.dataset.theme === "dark";
function setTheme(theme, save) {
  document.documentElement.dataset.theme = theme;
  document.querySelectorAll("[data-theme-toggle]").forEach((b) => b.setAttribute("aria-pressed", String(theme === "dark")));
  if (save) {
    try {
      localStorage.setItem("theme", theme);
    } catch {}
  }
}
document.addEventListener("click", (ev) => {
  if (ev.target.closest?.("[data-theme-toggle]")) setTheme(isDark() ? "light" : "dark", true);
});
if (!document.documentElement.dataset.theme) setTheme("light", false);

export function renderHeader(current) {
  const el = document.querySelector("[data-site-header]");
  if (!el) return;
  const link = (href, label, key) =>
    `<li><a href="${href}"${current === key ? ' aria-current="page"' : ""}>${label}</a></li>`;
  const themeLabel = t("Modo oscuro", "Dark mode", "深色模式");
  const aboutLabel = t("Acerca de · Contacto", "About us · Contact us", "关于我们 · 联系我们");
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="${HOME}">${LOGO()}<span class="brand-text"><strong>Mati Mati</strong><small>${t("Conjugador de lenguas indígenas", "Indigenous language conjugator", "原住民语言动词变位")}</small></span></a>
      <div class="header-end">
        <nav class="site-nav seg" aria-label="${t("Principal", "Main", "主导航")}">
          <ul>
            ${link(HOME, t("Inicio", "Home", "首页"), "home")}
            ${link(`${HOME}#lenguas`, t("Lenguas", "Languages", "语言"), "lenguas")}
            ${link(`${HOME}#mati`, "Mati Mati", "mati")}
          </ul>
        </nav>
        <span class="lang-switches seg" role="group" aria-label="${t("Idioma", "Language", "语言")}">${SWITCH.map((l) =>
          l.lang === LANG
            ? `<span class="lang-switch" aria-current="true" lang="${l.lang}">${l.label}</span>`
            : `<a class="lang-switch" data-lang-switch="${l.lang}" href="${languageHref(l.lang)}" hreflang="${l.lang}" lang="${l.lang}" aria-label="${l.name}">${l.label}</a>`
        ).join("")}</span>
        <span class="links">
          <button type="button" class="pill theme-toggle" data-theme-toggle aria-pressed="${isDark()}"
                  aria-label="${themeLabel}" title="${themeLabel}">${MOON_SVG}${SUN_SVG}</button>
          ${donateLink("pill donate")}
          <button type="button" class="pill about-btn" data-about-open aria-haspopup="dialog" title="${aboutLabel}"><span aria-hidden="true">i</span><span class="lbl">${aboutLabel}</span></button>
        </span>
      </div>
    </div>`;
  renderAbout();
}

// "About · Contact" dialog, as in chinamitl.
function renderAbout() {
  if (document.getElementById("about")) return;
  const dlg = document.createElement("dialog");
  dlg.className = "about";
  dlg.id = "about";
  dlg.setAttribute("aria-labelledby", "about-title");
  dlg.innerHTML = `
    <button type="button" class="close" data-about-close aria-label="${t("Cerrar", "Close", "关闭")}">×</button>
    <div class="about-logo">${LOGO(56, "")}</div>
    <h2 id="about-title">${t("Acerca de Mati Mati", "About Mati Mati", "关于 Mati Mati")}</h2>
    <p>${t(
      "Mati Mati es una comunidad y plataforma para aprender, preservar y difundir lenguas indígenas y minoritarias.",
      "Mati Mati is a community and platform for learning, preserving and sharing Indigenous and minority languages.",
      "Mati Mati 是一个学习、保护和传播原住民语言及少数民族语言的社区与平台。"
    )}</p>
    <p>${t(
      "El conjugador muestra, para cada lengua, quién la habla, cómo se escribe y cómo cambia un verbo con cada persona y cada tiempo.",
      "The conjugator shows, for each language, who speaks it, how it is written and how a verb changes with each person and tense.",
      "这个变位工具为每种语言介绍谁在说它、怎样书写，以及动词怎样随人称和时态变化。"
    )}</p>
    <h3>${t("Contáctanos", "Contact us", "联系我们")}</h3>
    <p>${t(
      "¿Tienes una corrección, una palabra de tu comunidad o una idea? Escríbenos:",
      "Have a correction, a word from your community or an idea? Write to us:",
      "有需要更正的内容、你所在社区的说法或新想法？欢迎来信："
    )}</p>
    <div class="email-row">
      <a class="email" href="mailto:${EMAIL}">${EMAIL}</a>
      <button type="button" class="copy" data-copy-email>${t("Copiar", "Copy", "复制")}</button>
    </div>
    <h3>${t("Apóyanos", "Support us", "支持我们")}</h3>
    <p>${t(
      "Si te gusta este proyecto, puedes apoyarnos con una donación para seguir creando contenido sobre nuestras lenguas.",
      "If you enjoy this project, you can support us with a donation so we can keep creating content about our languages.",
      "如果你喜欢这个项目，可以通过捐助支持我们继续创作有关这些语言的内容。"
    )}</p>
    <a class="donate-big" href="${DONATE_URL}" target="_blank" rel="noopener"><span aria-hidden="true">♥</span> ${t("Dona en Ko-fi", "Donate on Ko-fi", "在 Ko-fi 上捐助")}</a>`;
  document.body.append(dlg);
}

document.addEventListener("click", async (ev) => {
  const dlg = document.getElementById("about");
  if (!dlg) return;
  if (ev.target.closest?.("[data-about-open]")) dlg.showModal?.();
  else if (ev.target.closest?.("[data-about-close]") || ev.target === dlg) dlg.close();
  else if (ev.target.closest?.("[data-copy-email]")) {
    const btn = ev.target.closest("[data-copy-email]");
    try {
      await navigator.clipboard.writeText(EMAIL);
      btn.textContent = t("¡Copiado!", "Copied!", "已复制！");
    } catch {
      btn.textContent = EMAIL;
    }
  }
});

export function renderFooter() {
  const el = document.querySelector("[data-site-footer]");
  if (!el) return;
  el.innerHTML = `
    <div class="wrap">
      <a class="brand" href="${HOME}">${LOGO_WHITE()}<span class="brand-text"><strong>Mati Mati</strong></span></a>
      <p>${t("Aprende, preserva y comparte lenguas indígenas.", "Learn, preserve and share Indigenous languages.", "学习、保护和分享原住民语言。")}<br>
        ${t("Síguenos en", "Follow us at", "关注我们：")} <a href="https://www.instagram.com/matidemati" rel="noopener">@matidemati</a>.</p>
      ${donateLink("pill donate donate--footer")}
    </div>`;
}

// Keeps the language switch pointing at the same verb and section after the
// URL changes (?verbo=…, #conjugador).
export function refreshLanguageSwitch() {
  document
    .querySelectorAll("[data-lang-switch]")
    .forEach((a) => a.setAttribute("href", languageHref(a.dataset.langSwitch)));
}
window.addEventListener("hashchange", refreshLanguageSwitch);
