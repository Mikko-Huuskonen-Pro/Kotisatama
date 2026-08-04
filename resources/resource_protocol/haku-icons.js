(function (global) {
  "use strict";

  // KOTISATAMA-PATCH: inline-SVG (ei emoji-fonttia) — 内联SVG（无表情字体依赖）。
  // Paths use fill="currentColor" via CSS on the SVG root.
  const CATEGORY_SVGS = {
    emergency:
      '<path d="M8 1.5L1.5 14h13L8 1.5zm0 3.2l4.2 7.8H3.8L8 4.7zM7.3 8h1.4v2.5H7.3V8zm0 3.2h1.4V12.5H7.3v-1.3z"/>',
    government:
      '<path d="M8 1.5L2 4.5v1h12v-1L8 1.5zM3 6.5v5H2v1.5h12V11.5h-1v-5H3zm1.5 0h2v5h-2v-5zm3.5 0h2v5H8v-5zm3.5 0h2v5h-2v-5z"/>',
    municipality:
      '<path d="M2 13.5V6l3-2 3 2 3-2 3 2v7.5H2zm1.5-1.5h2V8.5h-2V12zm3.5 0h2V7H7v5zm3.5 0h2V8.5h-2V12z"/>',
    city:
      '<path d="M2 13.5V6l3-2 3 2 3-2 3 2v7.5H2zm1.5-1.5h2V8.5h-2V12zm3.5 0h2V7H7v5zm3.5 0h2V8.5h-2V12z"/>',
    health:
      '<path d="M8 14s-5.5-3.4-5.5-7A3.2 3.2 0 0 1 8 4.2 3.2 3.2 0 0 1 13.5 7c0 3.6-5.5 7-5.5 7z"/>',
    education:
      '<path d="M8 2.5L1.5 5.5 8 8.5l6.5-3L8 2.5zM3.2 7.2v3.3L8 12.8l4.8-2.3V7.2L8 9.5 3.2 7.2z"/>',
    library:
      '<path d="M3 2.5h1.8v11H3V2.5zm2.7 0H8v11H5.7V2.5zm3.2 0H14L12.2 13.5H8.9L10.7 2.5z"/>',
    transport:
      '<path d="M4 2.5h8a2 2 0 0 1 2 2v6.5H2V4.5a2 2 0 0 1 2-2zm-1 9.5h2.2v1.5H3V12zm7.8 0H13v1.5h-2.2V12zM4.5 4.5v3h7v-3h-7z"/>',
    banking:
      '<path d="M8 1.5L2 4.5v1.2h12V4.5L8 1.5zM3.2 6.7v5.3H2V13.5h12v-1.5h-1.2V6.7H3.2zm1.5 0h2v5.3h-2V6.7zm3.3 0h2v5.3H8V6.7zm3.3 0h2v5.3h-2V6.7z"/>',
    bank:
      '<path d="M8 1.5L2 4.5v1.2h12V4.5L8 1.5zM3.2 6.7v5.3H2V13.5h12v-1.5h-1.2V6.7H3.2zm1.5 0h2v5.3h-2V6.7zm3.3 0h2v5.3H8V6.7zm3.3 0h2v5.3h-2V6.7z"/>',
    commerce:
      '<path d="M2 3h1.6l.4 1.5h9.5l-1.2 5.5H5.2L4.7 8H3.2l.8 3.5h8.5L14.2 3H4.2L3.8 1.5H2V3zm3.5 10.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm5.5 0a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/>',
    shop:
      '<path d="M2 3h1.6l.4 1.5h9.5l-1.2 5.5H5.2L4.7 8H3.2l.8 3.5h8.5L14.2 3H4.2L3.8 1.5H2V3zm3.5 10.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm5.5 0a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/>',
    services:
      '<path d="M10.8 2.2l-.8 2.2 1.6 1.6 2.2-.8-.4 1.5-1.8.5.9 1.9-1.4.8-.9-1.9-1.5.6v2.1H7.3V9.4l-1.5-.6-.9 1.9-1.4-.8.9-1.9-1.8-.5-.4-1.5 2.2.8 1.6-1.6-.8-2.2 1.5-.3.8 1.9L8 3.8l.8 1.8.8-1.9 1.2.5zM7.2 11.5h1.6V14H7.2v-2.5z"/>',
    culture:
      '<path d="M3 4.5c0-1.5 1.5-2.5 3-2.5s2.5.7 2.5 2v7c0 .6-.4 1-1 1s-1-.4-1-1V7H5.5v4c0 1.5-1.2 2.5-2.5 2.5S.5 12.5.5 11V4.5h2.5zm8 0c0-1.5 1.5-2.5 3-2.5s2.5.7 2.5 2v7c0 .6-.4 1-1 1s-1-.4-1-1V7H13.5v4c0 1.5-1.2 2.5-2.5 2.5S8.5 12.5 8.5 11V4.5H11z"/>',
    sports:
      '<path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zm0 1.5c1 0 1.9.3 2.7.8L9.2 6.3 6.8 5.2 5.3 3.5C6.1 3.2 7 3 8 3zm-4.2 2.1l1.3 1.5-1 2.6-2.2.1A5 5 0 0 1 3.8 5.1zm.4 5.8l2.1-.1 1.1 2.4-.9 1.5A5 5 0 0 1 4.2 10.9zm5.1 3.7l.9-1.5 2.4.5A5 5 0 0 1 9.3 14.6zm3.3-2.5l-2.4-.5 1-2.6 2.1-1.1a5 5 0 0 1-.7 4.2zm.3-5.5l-2.1 1.1-1.1-2.4 1.5-2A5 5 0 0 1 12.9 6.6zM8 7.2l1.6.7-.6 1.6H7l-.6-1.6L8 7.2z"/>',
    nature:
      '<path d="M8 2c-2.5 2.5-3.5 4.5-3.5 6.2A3.5 3.5 0 0 0 8 11.7a3.5 3.5 0 0 0 3.5-3.5C11.5 6.5 10.5 4.5 8 2zM7.3 11.5V14h1.4v-2.5H7.3z"/>',
    work:
      '<path d="M6 3.5h4v1.5H6V3.5zM2.5 5.5h11v8H2.5v-8zm1.5 1.5v5h8v-5h-8z"/>',
    media:
      '<path d="M3 3h10v10H3V3zm1.5 1.5v3h7v-3h-7zm0 4.5v3h3v-3h-3zm4.5 0v3h2.5v-3H9z"/>',
    housing:
      '<path d="M8 2.5L2 7v6.5h4.5V9.5h3v4H14V7L8 2.5z"/>',
    home:
      '<path d="M8 2.5L2 7v6.5h4.5V9.5h3v4H14V7L8 2.5z"/>',
    religion:
      '<path d="M7.3 1.5h1.4v2.2h2.2v1.4H8.7v4.4c1.8.3 3.1 1.3 3.1 3v1.5H4.2V12.5c0-1.7 1.3-2.7 3.1-3V5.1H5.1V3.7h2.2V1.5z"/>',
    church:
      '<path d="M7.3 1.5h1.4v2.2h2.2v1.4H8.7v4.4c1.8.3 3.1 1.3 3.1 3v1.5H4.2V12.5c0-1.7 1.3-2.7 3.1-3V5.1H5.1V3.7h2.2V1.5z"/>',
    organization:
      '<path d="M5.5 6a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm5 0a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM2.5 13.5v-1c0-1.7 1.6-3 3.5-3h.8c.4 0 .8.1 1.2.2A3.4 3.4 0 0 0 7 11.5v2H2.5zm6 0v-2c0-.4.1-.8.3-1.2.5-.2 1.1-.3 1.7-.3h.8c1.9 0 3.5 1.3 3.5 3v1H8.5z"/>',
    auth:
      '<path d="M8 1.5A4.5 4.5 0 0 0 3.5 6v2.2H2.5v6.8h11V8.2h-1V6A4.5 4.5 0 0 0 8 1.5zm0 1.5A3 3 0 0 1 11 6v2.2H5V6a3 3 0 0 1 3-3zM8 9.5a1.5 1.5 0 0 1 .7 2.8V13.5H7.3v-1.2A1.5 1.5 0 0 1 8 9.5z"/>',
    other:
      '<path d="M7.2 11.5h1.6V13H7.2v-1.5zM8 2.5a4.2 4.2 0 0 1 4.2 4.2c0 1.5-.8 2.4-1.9 3.3-.7.6-1.1 1.1-1.1 1.8H7.6c0-1.3.6-2 1.5-2.8.9-.7 1.5-1.3 1.5-2.3A2.6 2.6 0 0 0 8 4.1 2.6 2.6 0 0 0 5.4 6.7H3.8A4.2 4.2 0 0 1 8 2.5z"/>',
  };

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function resolveCategoryIcon(categoryId, categories) {
    const fallback = CATEGORY_SVGS.other;
    if (!categoryId) {
      return svgIconSpan(fallback, "Muu");
    }
    const meta = (categories || []).find(function (item) {
      return item.id === categoryId;
    });
    const icon = meta ? meta.icon : categoryId;
    const svgInner = CATEGORY_SVGS[categoryId] || CATEGORY_SVGS[icon] || fallback;
    const label = meta && meta.label ? meta.label : categoryId;
    return svgIconSpan(svgInner, label);
  }

  function svgIconSpan(svgInner, label) {
    return (
      '<span class="category-icon category-svg" aria-label="' +
      escapeHtml(label) +
      '" title="' +
      escapeHtml(label) +
      '"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="100%" height="100%" aria-hidden="true" focusable="false">' +
      svgInner +
      "</svg></span>"
    );
  }

  function typeDot(typeId) {
    const css = typeId === "yellow" ? "yellow" : "white";
    const label = typeId === "yellow" ? "Keltainen" : "Valkoinen";
    return (
      '<span class="type-dot ' +
      css +
      '" aria-label="' +
      label +
      '" title="' +
      label +
      '"></span>'
    );
  }

  global.HakuIcons = {
    categoryIcon: resolveCategoryIcon,
    typeDot: typeDot,
  };
})(window);
