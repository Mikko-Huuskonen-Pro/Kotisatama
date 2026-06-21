(function (global) {
  "use strict";

  const DEFAULT_LOCALE = "fi";

  const STRINGS = {
    avomeri: {
      title: { fi: "Avomeri", sv: "Öppet hav" },
      heading: { fi: "Avomeri", sv: "Öppet hav" },
      intro: {
        fi: "Olet siirtymässä avomerelle. Kotisataman ulkopuolelle mennään vain käyttäjän omasta valinnasta.",
        sv: "Du håller på att gå ut på öppet hav. Utanför hemmahamnen sker det endast på användarens eget val.",
      },
      searchLabel: { fi: "Hae avomereltä", sv: "Sök på öppet hav" },
      searchPlaceholder: { fi: "Kirjoita hakusana…", sv: "Skriv sökord…" },
      searchButton: { fi: "Hae", sv: "Sök" },
      offline: {
        fi: "Myrsky: verkkoyhteyttä ei ole. Avomeri ei ole käytettävissä offline-tilassa.",
        sv: "Storm: ingen nätverksanslutning. Öppet hav är inte tillgängligt offline.",
      },
      pullopostiPrompt: {
        fi: 'Haluatko viestiä pullopostilla? <a href="servo:pulloposti">Avaa Pulloposti</a>',
        sv: 'Vill du skicka flaskpost? <a href="servo:pulloposti">Öppna Flaskpost</a>',
      },
    },
    pulloposti: {
      title: { fi: "Pulloposti", sv: "Flaskpost" },
      heading: { fi: "Pulloposti", sv: "Flaskpost" },
      intro: {
        fi: "Myrskymoodi: salattu viestintä ilman verkkoa. Kirjeet kulkevat laitteelta laitteelle Bluetoothin kautta — avain sovitaan etukäteen kuudella emojilla.",
        sv: "Stormläge: krypterad kommunikation utan nätverk. Brev färdas från enhet till enhet via Bluetooth — nyckeln avtalas i förväg med sex emojis.",
      },
      statusChecking: {
        fi: "Tarkistetaan Pulloposti-palvelua…",
        sv: "Kontrollerar Flaskpost-tjänsten…",
      },
      statusReady: {
        fi: "Pulloposti on valmiina. Voit lähettää ja vastaanottaa kirjeitä.",
        sv: "Flaskpost är redo. Du kan skicka och ta emot brev.",
      },
      statusError: {
        fi: "Pulloposti ei ole käynnissä. Käynnistä se Kotisatamasta uudelleen tai bundlaa pulloposti-daemon buildiin.",
        sv: "Flaskpost körs inte. Starta om från Kotisatama eller bunda in pulloposti-daemon i builden.",
      },
      openButton: { fi: "Avaa Pulloposti", sv: "Öppna Flaskpost" },
      retry: { fi: "Yritä uudelleen", sv: "Försök igen" },
      footer: {
        fi: "Pulloposti-prosessi käynnistyy taustalla (kuten paikallinen haku). Salauslogiikka pysyy laitteella — ei pilveä, ei tilejä.",
        sv: "Flaskpost-processen startar i bakgrunden (som den lokala sökningen). Krypteringslogiken stannar på enheten — inget moln, inga konton.",
      },
    },
    newtab: {
      title: { fi: "Kotisatama – uusi välilehti", sv: "Kotisatama – ny flik" },
      heading: { fi: "Kotisatama", sv: "Kotisatama" },
      searchPlaceholder: { fi: "Hae avomereltä…", sv: "Sök på öppet hav…" },
      searchButton: { fi: "Hae", sv: "Sök" },
    },
    config: {
      title: { fi: "Lisäasetukset", sv: "Avancerade inställningar" },
      searchPlaceholder: { fi: "Etsi asetusta…", sv: "Sök efter en inställning…" },
      searchAriaLabel: { fi: "Etsi asetuksia", sv: "Sök inställningar" },
      invalidInteger: {
        fi: "Anna kelvollinen kokonaisluku.",
        sv: "Ange ett giltigt heltal.",
      },
      languageSection: { fi: "Kieli", sv: "Språk" },
      languageLabel: { fi: "Kieliasetus", sv: "Språkinställning" },
      languageAuto: { fi: "Automaattinen (järjestelmä)", sv: "Automatiskt (system)" },
      languageFi: { fi: "Suomi", sv: "Finska" },
      languageSv: { fi: "Ruotsi", sv: "Svenska" },
    },
    license: {
      title: { fi: "Kolmannen osapuolen lisenssit", sv: "Tredjepartslicenser" },
      intro: {
        fi: 'Tämän ohjelman on tarjonnut <a href="https://servo.org/">Servo Project</a> <a href="https://www.mozilla.org/en-US/MPL/2.0/">Mozilla Public License Version 2.0 (MPL)</a> -lisenssillä. Servo Projectin lähdekoodi on MPL-lisensoitu. Muut kolmannen osapuolen komponentit on lisensoitu yhdellä tai useammalla ilmaisohjelman lisenssillä.',
        sv: 'Detta program har gjorts tillgängligt av <a href="https://servo.org/">Servo Project</a> under <a href="https://www.mozilla.org/en-US/MPL/2.0/">Mozilla Public License Version 2.0 (MPL)</a>. Servo Projects källkod är MPL-licensierad. Övriga tredjepartskomponenter licensieras under en eller flera fria programvarulicenser.',
      },
      allLicenseText: {
        fi: "Kaikki lisenssitekstit:",
        sv: "All licens-text:",
      },
    },
    blocked: {
      title: { fi: "Ei löydy kotisatamasta", sv: "Finns inte i hemmahamnen" },
      heading: {
        fi: "Tätä sivua ei löydy kotisatamassa.",
        sv: "Den här sidan finns inte i hemmahamnen.",
      },
      continueLink: { fi: "Jatka avomerelle", sv: "Fortsätt till öppet hav" },
      reportHint: {
        fi: 'Voit ilmoittaa ongelmasta tai ehdottaa sivustoa selaimen <strong>Ilmoita</strong>-napilla.',
        sv: 'Du kan anmäla ett problem eller föreslå en webbplats med webbläsarens knapp <strong>Anmäl</strong>.',
      },
    },
  };

  const LOCALE_STORAGE_KEY = "kotisatama.locale";

  function readStoredChoice() {
    try {
      return localStorage.getItem(LOCALE_STORAGE_KEY);
    } catch (_) {
      return null;
    }
  }

  function resolveLocaleCode(choice) {
    if (choice === "sv") {
      return "sv";
    }
    if (choice === "fi") {
      return "fi";
    }
    const langs = navigator.languages?.length
      ? navigator.languages
      : [navigator.language || DEFAULT_LOCALE];
    for (const lang of langs) {
      const code = String(lang).toLowerCase().split("-")[0];
      if (code === "sv") {
        return "sv";
      }
      if (code === "fi") {
        return "fi";
      }
    }
    return DEFAULT_LOCALE;
  }

  function detectLocale() {
    const stored = readStoredChoice();
    if (stored === "fi" || stored === "sv") {
      return stored;
    }
    return resolveLocaleCode(stored || "auto");
  }

  function getLocaleChoice() {
    const stored = readStoredChoice();
    if (stored === "fi" || stored === "sv" || stored === "auto") {
      return stored;
    }
    return "auto";
  }

  function setLocaleChoice(choice) {
    try {
      localStorage.setItem(LOCALE_STORAGE_KEY, choice);
    } catch (_) {
      /* private mode */
    }
    location.href =
      "servo:locale?set=" + encodeURIComponent(choice);
  }

  function t(page, key, locale) {
    const lang = locale || detectLocale();
    const entry = STRINGS[page]?.[key];
    if (!entry) {
      return key;
    }
    return entry[lang] ?? entry[DEFAULT_LOCALE] ?? key;
  }

  function applyDocument(page, locale) {
    const lang = locale || detectLocale();
    document.documentElement.lang = lang;

    const titleKey = document.querySelector("[data-i18n-title]");
    if (titleKey) {
      document.title = t(page, titleKey.getAttribute("data-i18n-title"), lang);
    }

    document.querySelectorAll("[data-i18n]").forEach((el) => {
      const key = el.getAttribute("data-i18n");
      el.textContent = t(page, key, lang);
    });

    document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
      const key = el.getAttribute("data-i18n-placeholder");
      el.placeholder = t(page, key, lang);
    });

    document.querySelectorAll("[data-i18n-aria-label]").forEach((el) => {
      const key = el.getAttribute("data-i18n-aria-label");
      el.setAttribute("aria-label", t(page, key, lang));
    });

    document.querySelectorAll("[data-i18n-html]").forEach((el) => {
      const key = el.getAttribute("data-i18n-html");
      el.innerHTML = t(page, key, lang);
    });

    return lang;
  }

  global.KotisatamaI18n = {
    DEFAULT_LOCALE,
    LOCALE_STORAGE_KEY,
    STRINGS,
    detectLocale,
    getLocaleChoice,
    setLocaleChoice,
    t,
    applyDocument,
  };
})(window);
