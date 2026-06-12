# Kotisatama — konseptitiivistelmä
*Versio 2.0 — kesäkuu 2026*

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
Meilisearch-indeksi laitteella (ei verkkoyhteyttä tarvita)
        ↓
Tulokset: kela.fi/eläke, eläkeliitto.fi/...
        ↓
Käyttäjä klikkaa — selain avaa sivun normaalisti
        ↓
Jos ei löydy: "Ei löydy kotisatamasta — haluatko mennä avomerelle?"
```

Indeksin koko: arviolta 50–200 MB per versio. Päivittyy OTA-päivityksen mukana viikoittain.

Fallback-haut ovat dataa: mitä whitelist kaipaa lisää.

---

## Tekninen arkkitehtuuri

### Selain
- Pohja: **Servo** (MPL 2.0 -lisenssi, vapaa forkkaus)
- Moottori: Servo — Rust-pohjainen, ei Googlen Chromium eikä Gecko
- Whitelist-logiikka lisätään suoraan Servo-komponentteihin
- Muutokset Servo-koodiin julkaistaan MPL:n mukaan — oma bisneslogiikka pysyy suljettuna

### UI-kerros
- **Tauri 2.0** wrappaa Servo-UI:n Android-applikaatioksi
- Web- ja mobiiliversio jakavat saman komponenttikirjaston

### Hakuindeksi
- **Meilisearch** (open source, Rust, virhesietoinen)
- Indeksi bundlataan laitteelle — ei palvelinriippuvuutta hakuun
- Crawler: **Playwright**-pohjainen (suorittaa JS:n → indeksoi SPA-sivustot oikein)
- Crawler pyörii CI-prosessina (esim. GitHub Actions), pushaa päivitetyn indeksin CDN:ään

### Infrastruktuuri — serverless
```
[Kotisatama-selain — Servo-fork + Tauri]
    ├── Whitelist JSON (bundlattu appiin, OTA-päivitys CDN:stä)
    └── Meilisearch-indeksi (ladattu laitteelle, päivittyy OTA)

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

- [ ] Servo-forkin rakenne — mihin whitelist-moduuli lisätään
- [ ] Hallintapaneelin toteutus ostajalle (web vai Tauri-app)
- [ ] Crawler-strategia: indeksointisyvyys, päivitystiheys
- [ ] Mainospaikan tekninen toteutus hakutuloksissa
- [ ] CDN-valinta (Cloudflare R2, Bunny, vai muu)
- [ ] Pro-julkaisun ajankohta — milloin ilmaisen dataa on riittävästi
- [ ] PRH: aputoiminimi tai uusi toiminimi Kotisatamalle

---

## Myyntipuheet

**Hopeakettu (aikuiselle lapselle):**
> *"Laitetaan äidille netti jossa ei ole haita."*

**Lapsi (vanhemmalle):**
> *"Internet ilman haita. Tee lapsen kanssa omat sivut."*

**Mainostajille:**
> *"Eksklusiivinen näkyvyys yleisölle joka ei ohita mainoksia."*

---

*Projekti on osa Ilio-toiminimeä (Y-tunnus 2010). Tekninen pohja: Servo (MPL 2.0) + Meilisearch (MIT) + Tauri 2.0 (MIT). Infrastruktuuri: serverless, CDN-pohjainen.*
