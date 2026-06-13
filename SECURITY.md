# Tietoturva (Security Policy)

## Haavoittuvuuksien ilmoittaminen

**Älä ilmoita tietoturva-aukkoja julkisena GitHub-issuena.**

Ilmoita haavoittuvuudet sähköpostitse:

**mikko@ilio.fi**

Vastaus 48 tunnin sisällä. Vakavat haavoittuvuudet pyritään korjaamaan 7 päivän sisällä ilmoituksesta.

---

## Mitä ilmoittaa

Erityisen kriittisiä Kotisatama-kontekstissa:

- Whitelist-ohitukset — tavat päästä whitelistan ulkopuolisille sivuille
- Sisällönsuodatuksen ohitus lapsiversiossa
- CDN-valheellinen whitelist (supply chain)
- Meilisearch-indeksin manipulointi

Servo-moottoriin liittyvät haavoittuvuudet voi ilmoittaa myös suoraan [servo/servo-repoon](https://github.com/servo/servo/security/advisories/new).

---

## Laajuus (Scope)

| Komponentti | Kuuluu tähän repoon |
|---|---|
| Whitelist-logiikka (`components/kotisatama/`) | ✅ Kyllä |
| Hakuindeksi-integraatio | ✅ Kyllä |
| CDN-päivitysmekanismi | ✅ Kyllä |
| Servo-selainmoottori (upstream) | ❌ Ilmoita servo/servo-repoon |

---

*Kotisatama on osa Ilio-toiminimeä (Y-tunnus 2010).*
