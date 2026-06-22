# VAIHE8-PULLOPOSTI.md — HTTP-API ja Android-bundlaus

*Suunnitelma. Julkisen repon toteutus etenee vaiheittain — ks. tilamerkinnät alla.*

**Tärkeä rajaus:** Varsinainen daemon-logiikka (BLE, salaus, avainten hallinta) asuu yksityisessä `pulloposti-daemon`-repossa. Tämä dokumentti määrittelee **rajapinnan** julkisen `kotisatama-pulloposti`-clientin ja daemonin välillä, sekä **Android-bundlauksen** vaiheet.

---

## Laajempi konteksti: ei vain Pulloposti

Suljettu repo on tarkoitettu kasvamaan useammaksi bundlatuksi subprocess-appiksi ajan myötä — Pulloposti on vasta ensimmäinen. Meilisearch ja Pulloposti seuraavat samaa kaavaa: HTTP-subprocess + health-check + binäärin paikannus/käynnistys + (Androidilla) `assets/`-bundlaus ja JNI-silta.

**Nyt on oikea hetki yleistää** tämä kaava yhteiseksi pohjaksi ennen kuin kolmas app kopioi saman logiikan uudestaan.

---

## 0. Yhteinen subprocess-app-pohja

- [x] Uusi crate `components/kotisatama/subprocess-app/` — binäärin paikannus, health-pollaus, `ManagedSubprocess` + `Drop`
- [x] `kotisatama-search` refaktoroitu käyttämään pohjaa
- [x] `kotisatama-pulloposti` refaktoroitu käyttämään pohjaa
- [ ] Android-puolelle vastaava yleistys: yksi `fetch-bundled-app.sh`-skripti app-nimelle + lähteelle
- [ ] JNI-nimeämiskäytäntö tuleville apeille: `kotisatama<AppNimi>Start`, `kotisatama<AppNimi>Health`, …

---

## Mikä on jo olemassa

| Osa | Tila |
|---|---|
| `kotisatama-subprocess-app` | Yhteinen subprocess-pohja |
| `kotisatama-pulloposti`-crate | Subprocess + HTTP-wrapperit (`/health`, `/peers`, `/pair`, `/letters`) |
| `servo:pulloposti`-gateway | Health-check + linkki app-näkymään |
| `servo:pulloposti/app` | `pulloposti-app.html` — kirjelista, lähetys, pariutuminen (polling) |
| Desktop-bundlaus | `scripts/sync-pulloposti-daemon.ps1` |
| `KOTISATAMA_PULLOPOSTI_BIN` / `KOTISATAMA_PULLOPOSTI_URL` | Käytössä |

**Puuttuu:**
- Daemonin toteutus suljetussa repossa (BLE, salaus)
- Android-bundlaus (cross-compile + assets + JNI)
- Push-malli uusille kirjeille (polling riittää MVP:hen)

---

## 1. HTTP-API-kontrakti (ehdotus)

Rajapinta jota julkinen client ja `pulloposti-app.html` kutsuvat:

```
GET  /health                    (jo olemassa)
GET  /peers                     -> lähistöllä olevat laitteet
POST /pair                      { "emoji_code": "🐟🌊⚓🔑🏠✉️" }
GET  /letters                   -> kirjeiden metadata
POST /letters                   { "to_peer_id": "...", "body": "..." }
GET  /letters/{id}              -> avattu kirje
DELETE /letters/{id}            -> poista paikallisesti
```

- [x] Julkisen clientin wrapper-funktiot (`kotisatama-pulloposti`)
- [ ] Vahvista kontrakti suljetussa repossa daemonin toteutuksen mukaan
- [ ] Virhetyypit: laajenna `PullopostiError` tarvittaessa (`PeerNotFound`, `PairingExpired`)

### Reaaliaikaisuus

- [x] **Polling**: UI kysyy `/letters`-listaa 5 s välein (`pulloposti-app.html`)
- [ ] **WebSocket/SSE**: myöhemmin jos akkukulutus nousee ongelmaksi

---

## 2. `servo:pulloposti/app` — sovellusnäkymä

- [x] `resources/resource_protocol/pulloposti-app.html` + `.css`
- [x] Reititys `servo:pulloposti/app` → `pulloposti-app.html`
- [x] Suomi/ruotsi `kotisatama-i18n.js`:n `pullopostiApp`-avaimilla

---

## 3. Android-bundlaus

- [ ] Cross-compile `pulloposti-daemon` → `aarch64-linux-android`
- [ ] `scripts/sync-pulloposti-daemon-android.sh` → `assets/kotisatama/bin/pulloposti-daemon`
- [ ] Runtime-kopio `assets/`:sta `filesDir`:iin + `chmod +x` (sama kuvio kuin Meilisearch)
- [ ] JNI: `kotisatamaPullopostiStart`, `kotisatamaPullopostiHealth`, …
- [ ] BLE-permissiot manifestissa

---

## Järjestysehdotus

1. ~~Eriytä yhteinen subprocess-app-pohja~~
2. Lukitse HTTP-API-kontrakti suljetussa repossa
3. Toteuta kontrakti daemonissa (suljettu repo)
4. ~~Laajenna julkinen client + app-näkymä~~
5. Android-cross-compile + bundlaus

---

## Avoimet päätökset

- [ ] Polling vai push uusille kirjeille
- [ ] Cross-compile-toolchain: `cargo-ndk` vai muu?
- [ ] Säilytetäänkö luetut kirjeet daemonissa pysyvästi?
- [ ] BLE-kantaman/akkuvaikutuksen testaus ennen julkaisua?

---

*Täydentää `ROADMAP-1.md`:n vaihetta 8. HTTP-API-kontrakti on lähtökohta — vahvista se suljetun repon ylläpitäjän kanssa ennen daemon-työtä.*
