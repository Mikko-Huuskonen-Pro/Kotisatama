# Kotisatama — konseptitiivistelmä
*Versio 2.1 — kesäkuu 2026*

---

## Visio

> **"Netti ilman haita."**

Kotisatama on whitelist-pohjainen selain ja esiladattu hakuverkko. Käyttäjä löytää kaiken tarvitsemansa ilman että eksyy avomerelle — eikä avomerta tarvita peruskäytössä lainkaan.

---

## Kohderyhmät

| Tuote | Käyttäjä | Ostaja | Kipupiste |
|---|---|---|---|
| Kotisatama (ilmainen) | Kaikki | Perhe | Huijaukset, eksyminen |
| Kotisatama Hopeakettu | Seniorit (65+) | Aikuinen lapsi | Huijaukset, yksinäisyys, eksyminen |
| Kotisatama Lapsi | Lapset (5–12v) | Vanhempi | Some, predatorit, satunnainen sisältö |

Kaikissa versioissa ostaja ja käyttäjä ovat **eri henkilö**. Myyntipuhe kohdistuu ostajalle.

---

## Go-to-market

```
Vaihe 1: Kotisatama (ilmainen)
    → Kerää käyttökokemusta
    → Fallback-lokit kertovat mitä haetaan

Vaihe 2: Kotisatama Hopeakettu + Kotisatama Lapsi (Pro)
    → Whitelistat rakennettu oikean datan päälle
    → Ei arvailua, vaan tieto siitä mitä käyttäjät oikeasti etsivät
```

---

## Kolme whitelistiä

| Versio | Whitelist | Hakuindeksin painotus |
|---|---|---|
| Kotisatama | Geneerinen, perhe hallinnoi itse | Yleinen |
| Hopeakettu | Ilio kuratoi: Kela, terveys, uutiset, reseptit, virastot | Suomenkielisyys, selkeä rakenne, iso fontti |
| Lapsi | Ilio kuratoi: oppiminen, pelit, harrastukset, ikäluokitettu sisältö | Kuvat, videot, ikäluokitus |

---

## Hakumalli — tuotteen ydin

Käyttäjä ei hae internetistä. Haku tapahtuu **laitteelle esiladatusta indeksistä**.

```
Käyttäjä kirjoittaa "eläke"
        ↓
Paikallinen Meilisearch-prosessi (bundlattu binääri, indeksi ladattu CDN:stä)
        ↓
Tulokset: kela.fi/eläke, eläkeliitto.fi/...
        ↓
Käyttäjä klikkaa — selain avaa sivun normaalisti
        ↓
Jos ei löydy: "Ei löydy kotisatamasta — haluatko mennä avomerelle?"
```

Haku ei vaatii verkkoyhteyttä, kun indeksi on ladattu. Meilisearch pyörii laitteella subprocessina (HTTP `127.0.0.1`) — ei kirjastotasolla upotettu.

Indeksin koko: arviolta 50–200 MB per versio. Päivittyy OTA-päivityksen mukana viikoittain.

Fallback-haut ovat dataa: mitä whitelist kaipaa lisää.

---

## Tekninen arkkitehtuuri

### Selain
- Pohja: **Servo** (MPL 2.0 -lisenssi, vapaa forkkaus)
- Moottori: Servo — Rust-pohjainen, ei Googlen Chromium eikä Gecko
- Whitelist-logiikka: `components/kotisatama/whitelist/` (oma crate)
- Whitelist-hook: `ports/servoshell/` embedder-kerros (`request_navigation` → allow/deny)
- Muutokset Servo-upstream-koodiin vain minimaalisia KOTISATAMA-PATCH-kohdilla; oma bisneslogiikka pysyy suljettuna

### UI-kerros
- **Desktop:** servoshell (egui-UI, hakukenttä)
- **Android:** servoshell EGL (`ports/servoshell/egl/android/` + `support/android/apk/`) — Servon oma JNI-polku
- **Tauri 2.0:** erillinen hallintapaneeli vanhemman whitelist-hallintaan (web/desktop) — **ei selainmoottori**
- Tauri käyttää Androidilla System WebViewia (Chromium) — se ei kantaa Servo-moottoria

### Hakuindeksi
- **Meilisearch** (open source, Rust, virhesietoinen) — palvelinprosessi laitteella
- Indeksi CDN:stä Meilisearch-dumpina; prosessi importaa dumpin käynnistyksessä
- `components/kotisatama/search/` — HTTP-client paikalliseen instanssiin, ei Meilisearch-core upotettu
- Crawler: **Playwright**-pohjainen (suorittaa JS:n → indeksoi SPA-sivustot oikein)
- Crawler pyörii CI-prosessina (esim. GitHub Actions), pushaa päivitetyn indeksin CDN:ään

### Infrastruktuuri — serverless
```
[Kotisatama-selain — Servo-fork + servoshell]
    ├── components/kotisatama/whitelist   ← logiikka
    ├── components/kotisatama/search    ← HTTP-client
    ├── ports/servoshell                ← navigointi + UI
    ├── Whitelist JSON (bundlattu appiin, OTA-päivitys CDN:stä)
    └── Meilisearch subprocess (bundlattu binääri + ladattu indeksi)

[Tauri 2.0 — erillinen hallintapaneeli]
    └── Vanhempi hallinnoi whitelistia (ei osa selainmoottoria)

[CDN — staattinen]
    ├── Whitelist JSON (versioitu)
    │   ├── /free/whitelist.json        ← ilmainen, julkinen
    │   └── /pro/whitelist.json         ← Pro, API-avain vaaditaan
    └── Meilisearch-indeksidumpit

[Crawler — CI-prosessi]
    └── Playwright indeksoi whitelist-sivustot → pushaa CDN:ään
```

Ei omaa palvelinta. Ei VPN-infraa. Ei sertifikaattihallintaa. Ei revokatiota.

Ops-taakka: lähes nolla. Kustannus: CDN-liikenne + crawler-CI.

### Pro-suojaus
Maksulliset whitelistat ja indeksit haetaan CDN-polusta joka vaatii API-avaimen. Avain tulee sovellukseen maksun vahvistuksen jälkeen.

---

## Liiketoimintamalli

### Tulokerrokset

| Kerros | Malli | Kenelle |
|---|---|---|
| Kotisatama | Ilmainen, perhe hallinnoi itse | Sisäänheittotuote, kokemusdatan keruu |
| Kotisatama Pro | 5€/kk, Ilio hallinnoi ja kuratoi | Hopeakettu + Lapsi -versiot |
| Hakumainonta | Paikallinen yritys maksaa näkyvyydestä | Kampaajat, apteekit, taksipalvelut jne. |

### Hakumainonta
- Yksi mainostaja per hakusana per paikkakunta
- Merkitty selkeästi: *"Suositeltu kotisatamassa"*
- Ilio hyväksyy jokaisen mainostajan manuaalisesti
- Ei datankeräystä, ei retargetointia — lähempänä keltaisia sivuja kuin Googlea
- Potentiaali: 500 yritystä × 20€/kk = 10 000€/kk

### Konversiologiikka
```
Ilmainen → kokemus karttuu → "en jaksa hallita itse" → 5€/kk Pro
```

---

## Mitä tämä estää

| Tilanne | Tulos |
|---|---|
| Vahingossa väärä osoite | Selain näyttää neutraalin virheen |
| Huijaussähköpostin linkki | Ei löydy whitelistiltä — blokki |
| Lapsi etsii sopimattomia hakusanoja | Ei osumia indeksistä |
| Satunnainen mainossisältö | Ei pääse indeksiin |

---

## Avoimet päätökset

- [x] Servo-forkin rakenne — `components/kotisatama/` + `ports/servoshell/` embedder-hook
- [x] Android-polku — servoshell EGL (ei Tauri selaimen kantajana)
- [x] Haku laitteella — Meilisearch subprocess + CDN-dump (ei kirjastotasolla upotettu)
- [ ] Hallintapaneeli — Tauri 2.0 hallintapaneeli (suositus), web-vaihtoehto mahdollinen
- [ ] Crawler-strategia: indeksointisyvyys, päivitystiheys
- [ ] Mainospaikan tekninen toteutus hakutuloksissa
- [ ] CDN-valinta (Cloudflare R2, Bunny, vai muu)
- [ ] Pro-julkaisun ajankohta — milloin ilmaisen dataa on riittävästi
- [ ] PRH: aputoiminimi tai uusi toiminimi Kotisatamalle

## Toteutusjärjestys (tekninen)

1. `components/kotisatama/whitelist` + `request_navigation`-hook servoshellissa
2. Hakukenttä servoshell-UI:ssa (desktop)
3. Crawler + CDN-pipeline
4. Meilisearch subprocess + `kotisatama-search`
5. Android: servoshell EGL (`./mach build --target aarch64-linux-android`)
6. Tauri-hallintapaneeli (valinnainen, erillinen)

---

## Myyntipuheet

**Hopeakettu (aikuiselle lapselle):**
> *"Laitetaan äidille netti jossa ei ole haita."*

**Lapsi (vanhemmalle):**
> *"Internet ilman haita. Tee lapsen kanssa omat sivut."*

**Mainostajille:**
> *"Eksklusiivinen näkyvyys yleisölle joka ei ohita mainoksia."*

---

*Projekti on osa Ilio-toiminimeä (Y-tunnus 2010). Tekninen pohja: Servo + servoshell (MPL 2.0) + Meilisearch subprocess (MIT) + Tauri 2.0 hallintapaneeli (MIT). Infrastruktuuri: serverless, CDN-pohjainen.*
