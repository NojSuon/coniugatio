import { renderHeader, renderFooter, langWord, escapeHtml } from "./common.js";

renderHeader("home");
renderFooter();

const grid = document.querySelector("[data-language-grid]");

function card(lang) {
  const available = lang.status === "available";
  const body = `
    <span class="tag">${available ? "Disponible" : "Próximamente"}</span>
    ${langWord(lang.endonym, "span", "endonym")}
    <span class="name">${escapeHtml(lang.variant)}</span>
    <span class="region">${escapeHtml(lang.region)}</span>
    <p class="summary">${escapeHtml(lang.summary)}</p>
    ${available ? `<span class="go">Conjugar →</span>` : ""}`;
  return available
    ? `<a class="language-card" data-serie="${lang.series}" href="${lang.slug}/">${body}</a>`
    : `<div class="language-card" data-serie="${lang.series}" aria-disabled="true">${body}</div>`;
}

try {
  const res = await fetch("data/languages.json");
  if (!res.ok) throw new Error(res.statusText);
  const languages = await res.json();
  grid.innerHTML = languages.map(card).join("");
} catch {
  grid.innerHTML = `<p class="error">No pudimos cargar las lenguas. Recarga la página para intentarlo de nuevo.</p>`;
}
