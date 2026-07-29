(function (global) {
  "use strict";

  const CATEGORY_EMOJIS = {
    emergency: "🚨",
    government: "🏛️",
    municipality: "🏙️",
    city: "🏙️",
    health: "❤️",
    education: "🎓",
    library: "📚",
    transport: "🚌",
    banking: "🏦",
    bank: "🏦",
    commerce: "🛒",
    shop: "🛒",
    services: "🛠️",
    culture: "🎭",
    sports: "⚽",
    nature: "🌲",
    work: "💼",
    media: "📰",
    housing: "🏠",
    home: "🏠",
    religion: "⛪",
    church: "⛪",
    organization: "🤝",
    other: "🔎",
  };

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function resolveCategoryEmoji(categoryId, categories) {
    const fallback = CATEGORY_EMOJIS.other;
    if (!categoryId) {
      return emojiSpan(fallback, "Muu");
    }
    const meta = (categories || []).find(function (item) {
      return item.id === categoryId;
    });
    const icon = meta ? meta.icon : categoryId;
    const emoji = CATEGORY_EMOJIS[categoryId] || CATEGORY_EMOJIS[icon] || fallback;
    const label = meta && meta.label ? meta.label : categoryId;
    return emojiSpan(emoji, label);
  }

  function emojiSpan(emoji, label) {
    return (
      '<span class="category-icon category-emoji" aria-label="' +
      escapeHtml(label) +
      '" title="' +
      escapeHtml(label) +
      '">' +
      emoji +
      "</span>"
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
    categoryIcon: resolveCategoryEmoji,
    typeDot: typeDot,
  };
})(window);
