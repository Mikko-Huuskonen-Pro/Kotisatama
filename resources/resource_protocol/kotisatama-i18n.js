(function (global) {
  "use strict";

  const DEFAULT_LOCALE = "fi";

  const STRINGS = {
    avomeri: {
      title: { fi: "Avomeri", sv: "Öppet hav" },
      heading: { fi: "Avomeri", sv: "Öppet hav" },
      intro: {
        fi: "Avomeri on erillinen poikkeustila avoimelle netille. Satama ei avaa sitä automaattisesti.",
        sv: "Öppet hav är ett separat undantagsläge för det öppna nätet. Hamnen öppnar det inte automatiskt.",
      },
      requestHeading: { fi: "Avomerelle pyydetty haku", sv: "Sökning begärd till öppet hav" },
      requestHelp: {
        fi: "Jatka vain, jos haluat poistua Sataman whitelist-suojasta.",
        sv: "Fortsätt bara om du vill lämna hamnens whitelist-skydd.",
      },
      openAvomeri: { fi: "Jatka Avomerelle", sv: "Fortsätt till öppet hav" },
      backSatama: { fi: "Takaisin Satamaan", sv: "Tillbaka till hamnen" },
      manageSites: { fi: "Ehdota tai lisää sivusto", sv: "Föreslå eller lägg till webbplats" },
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
    pullopostiApp: {
      title: { fi: "Pulloposti", sv: "Flaskpost" },
      heading: { fi: "Pulloposti", sv: "Flaskpost" },
      intro: {
        fi: "Lähetä ja vastaanota salattuja kirjeitä lähilaitteiden kautta. Kirjeet synkronoidaan taustalla.",
        sv: "Skicka och ta emot krypterade brev via närliggande enheter. Brev synkas i bakgrunden.",
      },
      statusReady: {
        fi: "Yhteys Pullopostiin toimii.",
        sv: "Anslutningen till Flaskpost fungerar.",
      },
      statusError: {
        fi: "Pulloposti ei vastaa. Tarkista että daemon on käynnissä.",
        sv: "Flaskpost svarar inte. Kontrollera att daemonen körs.",
      },
      lettersHeading: { fi: "Kirjeet", sv: "Brev" },
      lettersEmpty: { fi: "Ei kirjeitä vielä.", sv: "Inga brev ännu." },
      composeHeading: { fi: "Uusi kirje", sv: "Nytt brev" },
      toPeerLabel: { fi: "Vastaanottajan tunniste", sv: "Mottagarens ID" },
      bodyLabel: { fi: "Viesti", sv: "Meddelande" },
      sendButton: { fi: "Lähetä", sv: "Skicka" },
      sendSuccess: {
        fi: "Kirje lähetetty.",
        sv: "Brevet skickades.",
      },
      pairHeading: { fi: "Pariutuminen", sv: "Parkoppling" },
      pairIntro: {
        fi: "Syötä kuusi emojia, jotka olette sopineet etukäteen lähilaitteen kanssa.",
        sv: "Ange sex emojis som ni kommit överens om i förväg med en närliggande enhet.",
      },
      emojiLabel: { fi: "Emoji-koodi", sv: "Emoji-kod" },
      pairButton: { fi: "Aloita pariutuminen", sv: "Starta parkoppling" },
      refreshPeers: { fi: "Päivitä laitteet", sv: "Uppdatera enheter" },
      peerPaired: { fi: "Paritettu", sv: "Parkopplad" },
      peerUnpaired: { fi: "Ei paritettu", sv: "Inte parkopplad" },
      openLetter: { fi: "Avaa", sv: "Öppna" },
      deleteLetter: { fi: "Poista", sv: "Ta bort" },
      letterEmpty: { fi: "(tyhjä kirje)", sv: "(tomt brev)" },
      pairStarted: {
        fi: "Pariutuminen käynnistetty.",
        sv: "Parkoppling startad.",
      },
      backGateway: { fi: "Takaisin alkuun", sv: "Tillbaka till start" },
    },
    newtab: {
      title: { fi: "Kotisatama – uusi välilehti", sv: "Kotisatama – ny flik" },
      heading: { fi: "Kotisatama", sv: "Kotisatama" },
      searchPlaceholder: { fi: "Kirjoita osoite tai hae satamasta…", sv: "Skriv adress eller sök i hamnen…" },
      searchButton: { fi: "Hae", sv: "Sök" },
    },
    varustamo: {
      title: { fi: "Varustamo", sv: "Varustamo" },
      heading: { fi: "Varustamo", sv: "Varustamo" },
      intro: {
        fi: "Luotettu sovellusvarasto Kotisatamassa. Valitse sovellus alla.",
        sv: "Betrodd appbutik i Kotisatama. Välj en app nedan.",
      },
      loading: { fi: "Ladataan sovellusluetteloa…", sv: "Laddar applista…" },
      empty: { fi: "Ei asennettavia sovelluksia.", sv: "Inga appar att installera." },
      loadError: {
        fi: "Rekisteriä ei voitu ladata. Synkkaa suljetusta reposta.",
        sv: "Registret kunde inte laddas. Synka från det stängda repot.",
      },
      permAllowed: { fi: "Sallii:", sv: "Tillåter:" },
      permDenied: { fi: "Ei salli:", sv: "Tillåter inte:" },
      backLink: { fi: "Takaisin Kotisatamaan", sv: "Tillbaka till Kotisatama" },
    },
    missaOlen: {
      title: { fi: "Missä olen", sv: "Var är jag" },
      heading: { fi: "Missä olen", sv: "Var är jag" },
      intro: {
        fi: "Selvitä sijaintisi turvallisesti paikallisen daemonin kautta.",
        sv: "Ta reda på var du är via den lokala daemonen.",
      },
      statusChecking: {
        fi: "Tarkistetaan Missä olen -palvelua…",
        sv: "Kontrollerar Var är jag-tjänsten…",
      },
      statusReady: {
        fi: "Missä olen on valmiina.",
        sv: "Var är jag är redo.",
      },
      statusError: {
        fi: "Missä olen ei ole käynnissä. Käynnistä daemon Varustamosta tai synkkaa buildiin.",
        sv: "Var är jag körs inte. Starta daemonen från Varustamo eller synka till builden.",
      },
      locateButton: { fi: "Paikanna minut", sv: "Lokalisera mig" },
      locating: { fi: "Haetaan sijaintia…", sv: "Hämtar position…" },
      located: { fi: "Sijainti löytyi:", sv: "Position hittad:" },
      geoUnsupported: {
        fi: "Selaimesi ei tue paikannusta.",
        sv: "Din webbläsare stöder inte positionering.",
      },
      geoDenied: {
        fi: "Paikannus estettiin. Salli sijainti laitteen asetuksista.",
        sv: "Positionering nekades. Tillåt plats i enhetens inställningar.",
      },
      reverseError: {
        fi: "Osoitteen haku epäonnistui.",
        sv: "Adressökningen misslyckades.",
      },
      retry: { fi: "Yritä uudelleen", sv: "Försök igen" },
      backVarustamo: { fi: "Takaisin Varustamoon", sv: "Tillbaka till Varustamo" },
      attribution: {
        fi: "Osoitetiedot: OpenStreetMap / Nominatim.",
        sv: "Adressdata: OpenStreetMap / Nominatim.",
      },
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
      mySitesSection: { fi: "Sataman sivut", sv: "Hamnens sidor" },
      mySitesLink: {
        fi: "Hallitse omia sataman sivuja",
        sv: "Hantera egna sidor i hamnen",
      },
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
      title: { fi: "Ei löydy satamasta", sv: "Finns inte i hamnen" },
      heading: {
        fi: "Tätä sivua ei löydy satamassa.",
        sv: "Den här sidan finns inte i hamnen.",
      },
      addSiteLink: { fi: "Lisää satamaan", sv: "Lägg till i hamnen" },
      avomeriLink: { fi: "Avaa Avomeri-portti", sv: "Öppna porten till öppet hav" },
      manageSitesLink: { fi: "Omat sataman sivut", sv: "Egna sidor i hamnen" },
      reportHint: {
        fi: 'Voit kirjata havainnon lokikirjaan selaimen <strong>Lokikirja</strong>-napilla.',
        sv: 'Du kan anteckna i loggboken med webbläsarens knapp <strong>Loggbok</strong>.',
      },
    },
    mySites: {
      title: { fi: "Omat sataman sivut", sv: "Egna sidor i hamnen" },
      heading: { fi: "Omat sataman sivut", sv: "Egna sidor i hamnen" },
      intro: {
        fi: "Nämä sivut ovat vain tässä laitteessa. Ne eivät korvaa kuratoitua listaa eivätkä näy haussa ennen kuin ne on indeksoitu erikseen.",
        sv: "Dessa sidor finns bara på den här enheten. De ersätter inte den kurerade listan och syns inte i sökningen förrän de indexeras separat.",
      },
      empty: { fi: "Et ole vielä lisännyt omia sivuja.", sv: "Du har inte lagt till egna sidor än." },
      domainPlaceholder: { fi: "esim. oma-pizzeria.fi", sv: "t.ex. min-pizzeria.fi" },
      addButton: { fi: "Lisää sivu", sv: "Lägg till sida" },
      removeLink: { fi: "Poista", sv: "Ta bort" },
      addedOn: { fi: "Lisätty", sv: "Tillagd" },
      backLink: { fi: "Takaisin asetuksiin", sv: "Tillbaka till inställningar" },
      errorInvalid: {
        fi: "Domain ei kelpaa. Käytä muotoa esim. esimerkki.fi",
        sv: "Domänen är ogiltig. Använd t.ex. exempel.fi",
      },
      errorFailed: {
        fi: "Sivun lisääminen tai poistaminen epäonnistui.",
        sv: "Det gick inte att lägga till eller ta bort sidan.",
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
