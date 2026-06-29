# Linkit — upstream-lähteet

Kuratoitu lista Servon virallisista ja hyödyllisistä lähteistä. Lue ensin tiivistelmä oppimisdokumenteista, sitten alkuperäinen lähde syventävään lukemiseen.

## Servo Book (päälähde)

<https://book.servo.org>

| Luku | Aihe | Milloin lukea |
|------|------|---------------|
| [Setting up your environment](https://book.servo.org/hacking/setting-up-your-environment.html) | Asennus | Ennen ensimmäistä buildia |
| [Building Servo](https://book.servo.org/building/) | Käännös eri alustoille | Kun rakennat Androidia tai releasea |
| [Testing](https://book.servo.org/contributing/testing.html) | Testaus | Ennen WPT-ajoa |
| [Design documentation](https://book.servo.org/design-documentation/) | Suunnittelu | Kun syvennät arkkitehtuuriin |
| [Experimental features](https://book.servo.org/design-documentation/experimental-features.html) | Kokeelliset ominaisuudet | Kun tutkit prefsejä |

## Repot ja yhteisö

| Lähde | URL | Huomio |
|-------|-----|--------|
| Servo upstream | <https://github.com/servo/servo> | Alkuperäinen moottori |
| Servo Zulip | <https://servo.zulipchat.com> | Kehittäjäkeskustelu |
| Web Platform Tests | <https://web-platform-tests.org> | WPT-dokumentaatio |

## Standardit (web)

Kun debuggaat sivukohtaista bugia, vastaava spesifikaatio auttaa:

| Aihe | Lähde |
|------|-------|
| HTML | <https://html.spec.whatwg.org> |
| CSS | <https://drafts.csswg.org> |
| Fetch / HTTP | <https://fetch.spec.whatwg.org> |
| DOM | <https://dom.spec.whatwg.org> |

## Katselin-repon sisäiset linkit

| Dokumentti | Polku |
|------------|-------|
| Kehityssäännöt | [AGENT.md](../AGENT.md) |
| Filosofia | [docs/FILOSOFIA.md](../docs/FILOSOFIA.md) |
| Kela Telakka | [docs/KELA-TELAKKA.md](../docs/KELA-TELAKKA.md) |
| Komponenttikartta | [servo/komponentit.md](servo/komponentit.md) |

## `./mach`-komennot ja dokumentaatio

| Komento | Liittyvä book.servo.org -luku |
|---------|-------------------------------|
| `./mach build` | Building |
| `./mach run` | Hacking |
| `./mach test-wpt` | Testing |
| `./mach bootstrap` | Setting up your environment |

Tarkemmin: [servo/testaus-wpt.md](servo/testaus-wpt.md).
