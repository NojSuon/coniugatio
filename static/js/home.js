import { renderHeader, renderFooter, langWord, escapeHtml, t, PREFIX, DONATE_URL } from "./common.js";

renderHeader("home");
renderFooter();

const list = document.querySelector("[data-language-grid]");

// Names of the ids in the language data (family, area, continent), in
// Spanish, English and Chinese. An id missing here is shown as it is.
const FAMILIES = {
  otomangue: ["Otomangue", "Oto-Manguean", "奥托-曼格语系"],
  maya: ["Maya", "Mayan", "玛雅语系"],
  yutoazteca: ["Yutoazteca", "Uto-Aztecan", "犹他-阿兹特克语系"],
  mixezoque: ["Mixe-zoque", "Mixe-Zoquean", "米塞-索克语系"],
  aislada: ["Lengua aislada", "Language isolate", "孤立语言"],
  arawak: ["Arawak", "Arawakan", "阿拉瓦克语系"],
  tupi: ["Tupí-guaraní", "Tupi-Guarani", "图皮-瓜拉尼语族"],
  quechua: ["Quechua", "Quechuan", "克丘亚语系"],
  araucana: ["Araucana", "Araucanian", "阿劳坎语系"],
  coreanica: ["Coreánica", "Koreanic", "朝鲜语系"],
};
const AREAS = {
  mesoamerica: ["Mesoamérica", "Mesoamerica", "中部美洲"],
  sureste: ["Sureste de Norteamérica", "Southeastern North America", "北美东南部"],
  cascadia: ["Cascadia", "Cascadia", "卡斯卡迪亚"],
  caribe: ["Gran Caribe", "Greater Caribbean", "大加勒比地区"],
  plata: ["Cuenca del Plata", "Río de la Plata basin", "拉普拉塔河流域"],
  chaco: ["Gran Chaco", "Gran Chaco", "大查科地区"],
  andes: ["Andes", "Andes", "安第斯地区"],
  wallmapu: ["Wallmapu (Araucanía y Pampa)", "Wallmapu (Araucanía and the Pampas)", "瓦尔马普（阿劳卡尼亚与潘帕斯）"],
  asia_oriental: ["Asia oriental", "East Asia", "东亚"],
};
// Sections of the list, in order. `soon` is shown while a section is empty.
const CONTINENTS = [
  { id: "norte", name: t("Norteamérica y Centroamérica", "North and Central America", "北美洲和中美洲") },
  {
    id: "sur",
    name: t("Sudamérica", "South America", "南美洲"),
  },
  { id: "asia", name: t("Asia", "Asia", "亚洲") },
];
const label = (names, id) => (names[id] ? t(...names[id]) : id);

function card(lang) {
  const available = lang.status === "available";
  const body = `
    <span class="tag">${available ? t("Disponible", "Available", "可用") : t("Próximamente", "Coming soon", "即将推出")}</span>
    ${langWord(lang.endonym, "span", "endonym")}
    <span class="name">${escapeHtml(lang.variant)}</span>
    <span class="region">${escapeHtml(lang.region)}</span>
    <span class="family">${escapeHtml([label(FAMILIES, lang.family), label(AREAS, lang.area)].filter(Boolean).join(" · "))}</span>
    <p class="summary">${escapeHtml(lang.summary)}</p>
    ${available ? `<span class="go">${t("Conjugar", "Conjugate", "变位")} →</span>` : ""}`;
  const href = `${PREFIX}${lang.slug}/`;
  return available
    ? `<a class="language-card" data-serie="${lang.series}" href="${href}">${body}</a>`
    : `<div class="language-card" data-serie="${lang.series}" aria-disabled="true">${body}</div>`;
}

// A <select> with "all" and each value present in the data, with its count.
function select(key, names, title, all, languages) {
  const counts = new Map();
  for (const l of languages) if (l[key]) counts.set(l[key], (counts.get(l[key]) ?? 0) + 1);
  const options = [...counts]
    .map(([id, n]) => ({ id, n, name: label(names, id) }))
    .sort((a, b) => a.name.localeCompare(b.name));
  return `
    <label class="field">
      <span>${title}</span>
      <select data-filter="${key}">
        <option value="">${all}</option>
        ${options.map((o) => `<option value="${escapeHtml(o.id)}">${escapeHtml(o.name)} (${o.n})</option>`).join("")}
      </select>
    </label>`;
}

function render(languages) {
  list.innerHTML = `
    <div class="language-filters" role="group" aria-label="${t("Filtrar lenguas", "Filter languages", "筛选语言")}">
      ${select("family", FAMILIES, t("Familia lingüística", "Language family", "语系"), t("Todas las familias", "All families", "所有语系"), languages)}
      ${select("area", AREAS, t("Región", "Region", "地区"), t("Todas las regiones", "All regions", "所有地区"), languages)}
      <p class="caption" data-filter-count aria-live="polite"></p>
    </div>
    ${CONTINENTS.map((c) => {
      const own = languages.filter((l) => (l.continent || "norte") === c.id);
      return `
        <section class="continent" data-continent="${c.id}" aria-labelledby="continente-${c.id}">
          <h3 id="continente-${c.id}">${c.name}</h3>
          ${own.length ? `<div class="language-grid">${own.map((l) => card(l).replace(/^<(a|div) /, `<$1 data-family="${l.family}" data-area="${l.area}" `)).join("")}</div>` : ""}
          ${own.length || !c.soon ? "" : `<p class="caption continent-soon">${c.soon}</p>`}
        </section>`;
    }).join("")}
    <p class="caption" data-filter-empty hidden>${t(
      "Ninguna lengua coincide con estos filtros.",
      "No language matches these filters.",
      "没有符合这些条件的语言。"
    )} <button type="button" class="link-button" data-filter-reset>${t("Quitar filtros", "Clear filters", "清除筛选")}</button></p>`;

  const selects = [...list.querySelectorAll("[data-filter]")];
  const apply = () => {
    const want = Object.fromEntries(selects.map((s) => [s.dataset.filter, s.value]));
    const filtering = Object.values(want).some(Boolean);
    let shown = 0;
    for (const sec of list.querySelectorAll(".continent")) {
      let inSection = 0;
      for (const el of sec.querySelectorAll(".language-card")) {
        const ok = Object.entries(want).every(([k, v]) => !v || el.dataset[k] === v);
        el.hidden = !ok;
        if (ok) inSection++;
      }
      shown += inSection;
      // Empty sections are hidden while filtering (but "coming soon" stays otherwise).
      sec.hidden = filtering && inSection === 0;
    }
    list.querySelector("[data-filter-count]").textContent = filtering
      ? t(`${shown} de ${languages.length} lenguas`, `${shown} of ${languages.length} languages`, `${languages.length} 种语言中的 ${shown} 种`)
      : "";
    list.querySelector("[data-filter-empty]").hidden = shown > 0;
  };
  selects.forEach((s) => s.addEventListener("change", apply));
  list.querySelector("[data-filter-reset]").addEventListener("click", () => {
    selects.forEach((s) => (s.value = ""));
    apply();
    selects[0].focus();
  });
  apply();
}

// The hero carousel: the three newest languages (highest `order`) and a last
// slide asking for support on Ko-fi. It turns every 7 s, unless the reader
// prefers less motion, pauses it, or is pointing at or focused inside it.
function renderCarousel(languages) {
  const host = document.querySelector("[data-carousel]");
  if (!host) return;
  const newest = languages
    .filter((l) => l.status === "available")
    .sort((a, b) => b.order - a.order)
    .slice(0, 3);
  const total = newest.length + 1;
  const slideLabel = (i) => t(`${i + 1} de ${total}`, `${i + 1} of ${total}`, `第 ${i + 1} 张，共 ${total} 张`);
  const slides = newest.map((l, i) => `
    <a class="carousel-slide" data-serie="${l.series}" href="${PREFIX}${l.slug}/"
       role="group" aria-roledescription="${t("diapositiva", "slide", "幻灯片")}" aria-label="${slideLabel(i)}">
      <span class="tag">${t("Nueva lengua", "New language", "新语言")}</span>
      ${langWord(l.endonym, "span", "endonym")}
      <span class="name">${escapeHtml(l.name)}</span>
      <span class="summary">${escapeHtml(l.summary)}</span>
      <span class="go">${t("Conjugar", "Conjugate", "变位")} →</span>
    </a>`);
  slides.push(`
    <div class="carousel-slide carousel-slide--support"
         role="group" aria-roledescription="${t("diapositiva", "slide", "幻灯片")}" aria-label="${slideLabel(newest.length)}">
      <span class="tag">${t("Apóyanos", "Support us", "支持我们")}</span>
      <span class="support-title">${t("Ayúdanos a sumar más lenguas", "Help us add more languages", "帮助我们加入更多语言")}</span>
      <span class="summary">${t(
        "Mati Mati es gratuito. Si te sirve, un aporte en Ko-fi nos ayuda a seguir trabajando con más lenguas y comunidades.",
        "Mati Mati is free. If it helps you, a contribution on Ko-fi helps us keep working with more languages and communities.",
        "Mati Mati 是免费的。如果它对你有帮助，在 Ko-fi 上的支持能让我们继续与更多语言和社区合作。"
      )}</span>
      <a class="button donate" href="${DONATE_URL}" target="_blank" rel="noopener"><span aria-hidden="true">♥</span> ${t("Dona en Ko-fi", "Donate on Ko-fi", "在 Ko-fi 上捐助")}<span class="visually-hidden"> ${t("(se abre en otra pestaña)", "(opens in a new tab)", "（在新标签页打开）")}</span></a>
    </div>`);

  host.innerHTML = `
    <div class="carousel" aria-roledescription="${t("carrusel", "carousel", "轮播")}" aria-label="${t("Novedades", "What's new", "最新动态")}">
      <div class="carousel-track">${slides.join("")}</div>
      <div class="carousel-controls">
        <button type="button" class="carousel-btn" data-prev aria-label="${t("Anterior", "Previous", "上一张")}">‹</button>
        <div class="carousel-dots">
          ${slides.map((_, i) => `<button type="button" class="carousel-dot" data-go="${i}" aria-label="${t("Ir a la diapositiva", "Go to slide", "转到幻灯片")} ${i + 1}"></button>`).join("")}
        </div>
        <button type="button" class="carousel-btn" data-next aria-label="${t("Siguiente", "Next", "下一张")}">›</button>
        <button type="button" class="carousel-btn carousel-pause" data-pause></button>
      </div>
    </div>`;

  const track = host.querySelector(".carousel-track");
  const dots = [...host.querySelectorAll(".carousel-dot")];
  const pauseBtn = host.querySelector("[data-pause]");
  let current = 0;
  const show = (i, smooth = true) => {
    current = (i + total) % total;
    track.scrollTo({ left: current * track.clientWidth, behavior: smooth ? "smooth" : "auto" });
  };
  const mark = () => {
    dots.forEach((d, i) => d.setAttribute("aria-current", i === current ? "true" : "false"));
  };
  track.addEventListener("scroll", () => {
    const i = Math.round(track.scrollLeft / track.clientWidth);
    if (i !== current && i >= 0 && i < total) current = i;
    mark();
  }, { passive: true });
  window.addEventListener("resize", () => show(current, false));

  // Autoplay, off for readers who prefer less motion.
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  let playing = !reduced;
  let hold = false; // pointer over it or focus inside it
  const setPauseLabel = () => {
    pauseBtn.textContent = playing ? "❚❚" : "▶";
    pauseBtn.setAttribute("aria-label", playing
      ? t("Pausar el carrusel", "Pause the carousel", "暂停轮播")
      : t("Reanudar el carrusel", "Play the carousel", "播放轮播"));
  };
  setInterval(() => {
    if (playing && !hold && !document.hidden) show(current + 1);
  }, 7000);
  const stop = () => { playing = false; setPauseLabel(); };
  host.addEventListener("pointerenter", () => (hold = true));
  host.addEventListener("pointerleave", () => (hold = false));
  host.addEventListener("focusin", () => (hold = true));
  host.addEventListener("focusout", () => (hold = false));
  host.querySelector("[data-prev]").addEventListener("click", () => { stop(); show(current - 1); });
  host.querySelector("[data-next]").addEventListener("click", () => { stop(); show(current + 1); });
  dots.forEach((d) => d.addEventListener("click", () => { stop(); show(Number(d.dataset.go)); }));
  pauseBtn.addEventListener("click", () => { playing = !playing; setPauseLabel(); });
  track.addEventListener("touchstart", stop, { passive: true });
  setPauseLabel();
  mark();
}

try {
  const res = await fetch(`data/${PREFIX}languages.json`);
  if (!res.ok) throw new Error(res.statusText);
  const languages = await res.json();
  render(languages);
  try { renderCarousel(languages); } catch (e) { console.error(e); }
} catch {
  list.innerHTML = `<p class="error">${t(
    "No pudimos cargar las lenguas. Recarga la página para intentarlo de nuevo.",
    "We couldn't load the languages. Reload the page to try again.",
    "无法加载语言列表。请刷新页面重试。"
  )}</p>`;
}
