(function (global) {
  "use strict";

  // KOTISATAMA-PATCH: SVG-ikonit emoji-salasanalle (Android ilman emoji-fonttia).
  // Backend tallentaa Unicode-merkit; UI näyttää SVG:t samalla char-arvolla.
  const EMOJI_ICONS = [
    {
      id: "lock",
      char: "\u{1F512}",
      label: { fi: "Lukko", sv: "Lås" },
      svg: '<path d="M8 1.5A4.5 4.5 0 0 0 3.5 6v2.2H2.5v6.8h11V8.2h-1V6A4.5 4.5 0 0 0 8 1.5zm0 1.5A3 3 0 0 1 11 6v2.2H5V6a3 3 0 0 1 3-3zM8 9.5a1.5 1.5 0 0 1 .7 2.8V13.5H7.3v-1.2A1.5 1.5 0 0 1 8 9.5z"/>',
    },
    {
      id: "fox",
      char: "\u{1F98A}",
      label: { fi: "Kettu", sv: "Räv" },
      svg: '<path d="M4.5 12.5L3 10l2-4 3-.5L8 3l2.5 2.5L14 6l2 4-1.5 2.5H4.5zm1.8-1.5h4.4L12 10.2l-1.2-2.3-2.8-.8L5.2 7.9 4 10.2l2.3 1.8zM6.2 9.2h3.6L8 6.8 6.2 9.2z"/>',
    },
    {
      id: "anchor",
      char: "\u2693",
      label: { fi: "Ankkuri", sv: "Ankare" },
      svg: '<path d="M8 2.5v2.2M6 4.7h4M8 4.7v8.3M5.2 10.2a3.8 3.8 0 0 0 5.6 0M4.5 13.5h7"/>',
    },
    {
      id: "home",
      char: "\u{1F3E0}",
      label: { fi: "Koti", sv: "Hem" },
      svg: '<path d="M8 2.5L2 7v6.5h4.5V9.5h3v4H14V7L8 2.5z"/>',
    },
    {
      id: "star",
      char: "\u{1F31F}",
      label: { fi: "Tähti", sv: "Stjärna" },
      svg: '<path d="M8 1.8l1.8 3.7 4.1.6-3 2.9.7 4.1L8 11.2 4.4 13.1l.7-4.1-3-2.9 4.1-.6L8 1.8z"/>',
    },
    {
      id: "book",
      char: "\u{1F4DA}",
      label: { fi: "Kirja", sv: "Bok" },
      svg: '<path d="M3 2.5h1.8v11H3V2.5zm2.7 0H8v11H5.7V2.5zm3.2 0H14L12.2 13.5H8.9L10.7 2.5z"/>',
    },
    {
      id: "shield",
      char: "\u{1F6E1}",
      label: { fi: "Kilpi", sv: "Sköld" },
      svg: '<path d="M8 1.5L3 3.5v4.2c0 3.2 2.2 5.8 5 6.8 2.8-1 5-3.6 5-6.8V3.5L8 1.5z"/>',
    },
    {
      id: "wave",
      char: "\u{1F30A}",
      label: { fi: "Aalto", sv: "Våg" },
      svg: '<path d="M2 9.5c1.5-1 2.5-1 4 0s2.5 1 4 0 2.5-1 4 0M2 12c1.5-1 2.5-1 4 0s2.5 1 4 0 2.5-1 4 0"/>',
    },
    {
      id: "key",
      char: "\u{1F511}",
      label: { fi: "Avain", sv: "Nyckel" },
      svg: '<path d="M5.5 8.5a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5zm0-1.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2zM7.5 7.5h6.5v1.5H10v2H8.5v-2H7.5z"/>',
    },
    {
      id: "smile",
      char: "\u{1F60A}",
      label: { fi: "Hymy", sv: "Leende" },
      svg: '<path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zm0 1.5a5 5 0 1 1 0 10 5 5 0 0 1 0-10zM5.5 7a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm5 0a1 1 0 1 0 0-2 1 1 0 0 0 0 2zM5.8 10.2a3.2 3.2 0 0 0 4.4 0l.8 1.2a4.5 4.5 0 0 1-6 0l.8-1.2z"/>',
    },
    {
      id: "cat",
      char: "\u{1F431}",
      label: { fi: "Kissa", sv: "Katt" },
      svg: '<path d="M5 3.5L6.5 5 8 3.5 9.5 5 11 3.5 12.5 6.5 13.5 12.5H2.5L3.5 6.5 5 3.5zM6 9.5h1.5v1.5H6V9.5zm2.5 0H10v1.5H8.5V9.5zM6.5 11.5h3v1.5h-3v-1.5z"/>',
    },
    {
      id: "tree",
      char: "\u{1F333}",
      label: { fi: "Puu", sv: "Träd" },
      svg: '<path d="M8 2c-2.5 2.5-3.5 4.5-3.5 6.2A3.5 3.5 0 0 0 8 11.7a3.5 3.5 0 0 0 3.5-3.5C11.5 6.5 10.5 4.5 8 2zM7.3 11.5V14h1.4v-2.5H7.3z"/>',
    },
  ];

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function labelFor(icon, locale) {
    const lang = locale === "sv" ? "sv" : "fi";
    return (icon.label && icon.label[lang]) || icon.id;
  }

  function svgButtonHtml(icon) {
    return (
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="100%" height="100%" aria-hidden="true" focusable="false">' +
      icon.svg +
      "</svg>"
    );
  }

  function charsFromSelection(selectedIds) {
    return selectedIds
      .map(function (id) {
        const icon = EMOJI_ICONS.find(function (item) {
          return item.id === id;
        });
        return icon ? icon.char : "";
      })
      .filter(Boolean);
  }

  function renderPicker(container, options) {
    const locale = (options && options.locale) || "fi";
    const onChange = (options && options.onChange) || function () {};
    const selectedDisplay = (options && options.selectedDisplay) || null;
    let selected = [];

    container.innerHTML = "";
    EMOJI_ICONS.forEach(function (icon) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "emoji-btn emoji-svg-btn";
      btn.dataset.iconId = icon.id;
      btn.title = labelFor(icon, locale);
      btn.setAttribute("aria-label", labelFor(icon, locale));
      btn.innerHTML = svgButtonHtml(icon);
      btn.onclick = function () {
        if (selected.length >= 3) {
          selected = [];
        }
        selected.push(icon.id);
        const chars = charsFromSelection(selected);
        if (selectedDisplay) {
          selectedDisplay.textContent = chars.join(" ");
        }
        onChange(chars);
      };
      container.appendChild(btn);
    });

    return {
      reset: function () {
        selected = [];
        if (selectedDisplay) {
          selectedDisplay.textContent = "";
        }
        onChange([]);
      },
    };
  }

  global.KotisatamaEmojiIcons = {
    EMOJI_ICONS: EMOJI_ICONS,
    charsFromSelection: charsFromSelection,
    renderPicker: renderPicker,
  };
})(typeof window !== "undefined" ? window : globalThis);
