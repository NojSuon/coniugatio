import { renderHeader, renderFooter, langWord, escapeHtml, t, PREFIX } from "./common.js";

renderHeader("home");
renderFooter();

const grid = document.querySelector("[data-language-grid]");

function card(lang) {
  const available = lang.status === "available";
  const body = `
    <span class="tag">${available ? t("Disponible", "Available", "可用") : t("Próximamente", "Coming soon", "即将推出")}</span>
    ${langWord(lang.endonym, "span", "endonym")}
    <span class="name">${escapeHtml(lang.variant)}</span>
    <span class="region">${escapeHtml(lang.region)}</span>
    <p class="summary">${escapeHtml(lang.summary)}</p>
    ${available ? `<span class="go">${t("Conjugar", "Conjugate", "变位")} →</span>` : ""}`;
  const href = `${PREFIX}${lang.slug}/`;
  return available
    ? `<a class="language-card" data-serie="${lang.series}" href="${href}">${body}</a>`
    : `<div class="language-card" data-serie="${lang.series}" aria-disabled="true">${body}</div>`;
}

try {
  const res = await fetch(`data/${PREFIX}languages.json`);
  if (!res.ok) throw new Error(res.statusText);
  const languages = await res.json();
  grid.innerHTML = languages.map(card).join("");
} catch {
  grid.innerHTML = `<p class="error">${t(
    "No pudimos cargar las lenguas. Recarga la página para intentarlo de nuevo.",
    "We couldn't load the languages. Reload the page to try again.",
    "无法加载语言列表。请刷新页面重试。"
  )}</p>`;
}
