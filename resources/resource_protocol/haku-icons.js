(function (global) {
  "use strict";

  const CATEGORY_SVGS = {
    emergency:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2 3 20h18L12 2zm0 4.2 6.2 12.8H5.8L12 6.2zM11 10v4h2v-4h-2zm0 6v2h2v-2h-2z"/></svg>',
    government:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 21V9l9-5 9 5v12H3zm2-2h14v-9.3l-7-3.9-7 3.9V19zm3-2h2v-2H8v2zm4 0h2v-2h-2v2zm4 0h2v-2h-2v2zM8 13h2v-2H8v2zm4 0h2v-2h-2v2zm4 0h2v-2h-2v2z"/></svg>',
    city:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3 21V8l6-3 6 3v13H3zm2-2h2v-4H5v4zm4 0h2v-4H9v4zm4 0h2v-4h-2v4zm4 0h2v-4h-2v4zM8 11h2V9H8v2zm4 0h2V9h-2v2zm4 0h2V9h-2v2z"/></svg>',
    health:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 21s-7-4.4-9.5-8.6C.7 9.2 2.4 6 5.7 6c1.8 0 3.2.9 4.3 2.1C11.1 6.9 12.5 6 14.3 6 17.6 6 19.3 9.2 21.5 12.4 19 16.6 12 21 12 21z"/></svg>',
    education:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 3 1 9l11 6 9-4.9V17h2V9L12 3zm-7 9.2V17l7 4 7-4v-4.8l-7 3.8-7-3.8z"/></svg>',
    library:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M4 4h7v16H4V4zm9 0h7v16h-7V4zM6 6v12h3V6H6zm9 0v12h3V6h-3z"/></svg>',
    transport:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M5 17a2 2 0 1 0 .001 3.001A2 2 0 0 0 5 17zm12 0a2 2 0 1 0 .001 3.001A2 2 0 0 0 17 17zM6 5h9l3 4v6H5V5h1zm1 2v5h10V8l-2-1H7z"/></svg>',
    bank:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2 2 7v2h20V7L12 2zm-7 9v9h2v-9H5zm5 0v9h2v-9h-2zm5 0v9h2v-9h-2zm5 0v9h2v-9h-2zM2 20h20v2H2v-2z"/></svg>',
    shop:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M7 4h10l1 3h3v2l-2 9H5L3 9V7h3l1-3zm2.2 2 -.4 1h6.4l-.4-1H9.2zM6 11h12l1-3H5l1 3z"/></svg>',
    services:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.8-3.8a1 1 0 0 0-1.4-1.4l-3.1 3.1-1-1 3.1-3.1a1 1 0 0 0-1.4-1.4L14.7 6.3zM3 17.2V21h3.8L19.8 8l-3.8-3.8L3 17.2z"/></svg>',
    culture:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2a5 5 0 0 1 5 5c0 2.1-1.3 3.9-3.1 4.7V14h4v2H5v-2h4v-2.3C7.3 10.9 6 9.1 6 7a5 5 0 0 1 6-5zm0 2a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/></svg>',
    sports:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm-1 3.1c2 .4 3.7 1.7 4.6 3.5L12 10.2 8.4 8.6c.9-1.8 2.6-3.1 4.6-3.5zM6.2 9.3 10 11v5.3c-2.1-1.4-3.5-3.7-3.8-6.3v-.7zm11.6 0v.7c-.3 2.6-1.7 4.9-3.8 6.3V11l3.8-1.7z"/></svg>',
    nature:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2C8 7 5 9.5 5 13a7 7 0 0 0 14 0c0-3.5-3-6-7-11zm0 18c-2.8 0-5-2.2-5-5h10c0 2.8-2.2 5-5 5z"/></svg>',
    work:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M10 4h4v2h5v14H5V6h5V4zm2 2V6h0v2zM7 8v10h10V8H7zm2 2h2v2H9v-2zm4 0h2v2h-2v-2z"/></svg>',
    media:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M4 5h16v10H4V5zm2 2v6h12V7H6zm-2 9h20v2H2v-2z"/></svg>',
    home:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 3 2 12h3v8h6v-5h2v5h6v-8h3L12 3z"/></svg>',
    church:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2 9 5H7v3H5v14h14V8h-2V5h-4l-3-3zm0 4.8L13.2 8H10.8L12 6.8zM7 10h10v10H7V10zm2 2v2h2v-2H9zm4 0v2h2v-2h-2z"/></svg>',
    organization:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M16 11c1.7 0 3-1.3 3-3s-1.3-3-3-3-3 1.3-3 3 1.3 3 3 3zM8 11c1.7 0 3-1.3 3-3S9.7 5 8 5 5 6.3 5 8s1.3 3 3 3zm0 2c-2.7 0-8 1.3-8 4v3h16v-3c0-2.7-5.3-4-8-4zm8 0c-.3 0-.7 0-1 .1 1.2.8 2 1.9 2 3.4v2.5h6v-3c0-2.7-5.3-4-7-4z"/></svg>',
    other:
      '<svg class="category-icon" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M10 2a8 8 0 1 0 4.3 14.7l4.4 4.4 1.4-1.4-4.4-4.4A8 8 0 0 0 10 2zm0 2a6 6 0 1 1 0 12A6 6 0 0 1 10 4z"/></svg>',
  };

  const CATEGORY_ALIASES = {
    municipality: "city",
    banking: "bank",
    commerce: "shop",
    housing: "home",
    religion: "church",
  };

  function resolveCategoryIcon(categoryId, categories) {
    if (!categoryId) {
      return CATEGORY_SVGS.other;
    }
    const meta = (categories || []).find(function (item) {
      return item.id === categoryId;
    });
    const icon = meta ? meta.icon : CATEGORY_ALIASES[categoryId] || categoryId;
    return CATEGORY_SVGS[icon] || CATEGORY_SVGS.other;
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
