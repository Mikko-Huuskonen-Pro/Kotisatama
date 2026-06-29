# Oppimispäiväkirja

Lyhyet merkinnät siitä, mitä opit debugatessasi tai lukiessasi Servo-koodia. Tämä on **henkilökohtainen / tiimin oppimisloki**, ei virallinen bugiseuranta.

## Milloin kirjoittaa merkintä

- Ensimmäinen kerta kun ymmärrät komponentin roolin
- Kun löydät WPT-testin joka kuvaa saman ongelman
- Kun teet upstream-korjauksen tai opit miksi et voinut tehdä sitä vielä
- Kun termi selkenee — linkitä [sanasto.md](../../sanasto.md)

## Tiedostonimi

```
YYYY-MM-DD-lyhyt-aihe.md
```

Esimerkki: `2026-06-29-fetch-perusteet.md`

## Malli

Kopioi alla oleva pohja uuteen tiedostoon:

```markdown
# Otsikko — lyhyt kuvaus

**Päivä:** YYYY-MM-DD  
**Konteksti:** esim. Kela-etusivu, WPT-harjoitus  
**Komponentti:** esim. net, layout  

## Mitä yritin

- ...

## Mitä opin

- ...
- Linkki koodiin: `components/.../tiedosto.rs` (moduuli/ funktio, ei välttämättä rivinumeroa)

## Avoimet kysymykset

- ...

## Seuraava askel

- [ ] ...
```

## Esimerkkimerkinnät

- [2026-06-29-kela-etusivu.md](2026-06-29-kela-etusivu.md) — navigointi (`kela` / `kela.fi`), kerrosten erottelu, MVP-tarkistus

## Liittyvät dokumentit

- [miten-debugataan.md](../miten-debugataan.md)
- [docs/KELA-TELAKKA.md](../../../docs/KELA-TELAKKA.md)
