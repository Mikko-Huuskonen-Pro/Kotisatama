# Paikallinen testi-indeksi (vaihe 2)

Tiedosto `documents.json` seedataan Meilisearch-indeksiin ensimmäisellä käynnistyksellä,
kun indeksi on tyhjä.

## Vaatimukset

1. Asenna [Meilisearch](https://www.meilisearch.com/docs/learn/getting_started/installation)
   tai aseta `KOTISATAMA_MEILISEARCH_BIN` polku binääriin.
2. Käynnistä Kotisatama-selain (`./mach run`) — subprocess käynnistyy automaattisesti
   tai liitä jo käynnissä oleva instanssi osoitteessa `http://127.0.0.1:7700`.

## Ympäristömuuttujat

| Muuttuja | Oletus |
|---|---|
| `KOTISATAMA_MEILISEARCH_BIN` | `meilisearch` PATHissa |
| `KOTISATAMA_MEILISEARCH_DB` | `index-data/meilisearch` |
| `KOTISATAMA_MEILISEARCH_URL` | `http://127.0.0.1:7700` |
| `KOTISATAMA_SEARCH_DOCUMENTS` | `config/search-index/documents.json` |

## Testi

1. Avaa selain, kirjoita hakukenttään **eläke**, paina Enter.
2. Valitse tulos (esim. Kela) — sivu avautuu.
3. Hae jotain mitä indeksissä ei ole → *Hae avomereltä* avaa Startpage.
