# Różnice algorytmiczne: GPM (Strachey) — CPL vs Rust

Dokument zestawia różnice między algorytmem w wersji CPL (Appendix 2) a implementacją w Rust w projekcie.

Źródła:
- Oryginał CPL: `GPM_Appendix2_Strachey_CPL.txt`
- Implementacja Rust: `src/main.rs`

## Różnice w algorytmie (CPL vs Rust)

Poniżej porównanie logiki sterowania i semantyki kroków (nie samej składni języka).

- **Inny znak „definicji” (wejście do `Fn`)**
  - CPL mapuje znak `§` na akcję `Fn` w stanie `Start`.
  - Rust ustawia `CH_DEF` na `'&'` i tym znakiem uruchamia `Fn`.
  - Skutek: dla identycznego strumienia wejścia jak w oryginale (z `§`) Rust nie wejdzie w definicję makra (chyba że wejście jest już „przekodowane” na `&`).

- **Obsługa `~` (`LoadArg`) w `Start` jest uproszczona – zanika `Monitor2`**
  - CPL: w `Start` `~` zawsze kieruje do `LoadArg`, a `LoadArg` ma rozgałęzienie `P=0` → (`H=0` → `Copy`) lub `Monitor2`.
  - Rust: w `Start`, gdy `p==0`, od razu wybiera `Copy` i nie ma ścieżki do `Monitor2`.
  - Skutek: przypadek „niecytowanego `~` w argument list w strumieniu wejściowym” (Monitor2 w CPL) nie zostanie w Rust wykryty/obsłużony zgodnie z oryginałem.

- **EOF/`Marker`: Rust kończy program tam, gdzie CPL kieruje do `EndFn`/monitorów**
  - Rust zwraca `MARKER` jako sygnał EOF z `ReadSymbol` i w `Start` ma specjalny przypadek: jeśli `h==0 && c==0`, przechodzi do `Finish`.
  - CPL w `Start` traktuje `Marker` jako przejście do `EndFn`, a sytuacje „terminatora w strumieniu wejściowym” opisuje jako błąd (Monitor5).
  - Skutek: Rust ma inną semantykę „koniec danych” (EOF) niż wynika z przepływu sterowania w CPL.

- **Różnica w logice cytowania w `Q2` względem tekstu CPL**
  - W dostarczonym pliku CPL fragment `Q2` wygląda nietypowo (inkrementacja `q` na `>` zamiast na `<`).
  - Rust implementuje symetryczne zagnieżdżanie: `<` zwiększa `q`, `>` zmniejsza `q`.
  - Skutek: jeśli trzymać się literalnie dostarczonego pliku CPL, automat cytowania jest inny; możliwe, że to błąd/artefakt transkrypcji w CPL.

- **Makro `BAR`: inny wybór operacji „reszta” (`Rem`)**
  - CPL: gałąź domyślna wygląda jak `Rem[W,A]`.
  - Rust: resztę wykonuje tylko dla operatora `'R'`; dla innych nieznanych znaków robi błąd (`Monitor11`).
  - Skutek: dla operatorów spoza `+ - x /` (które w CPL wpadałyby w `Rem`) Rust zachowuje się inaczej.

- **Konwersja cyfr (`Number`/`Char`) ma inną definicję (zmienia „alfabet” danych)**
  - CPL deklaruje `Number[x] = x - 16` oraz `Char[x] = x + 16`.
  - Rust używa ASCII/Unicode: `number(x) = x - '0'` oraz generuje cyfry jako `'0' + q`.
  - Skutek: algorytmicznie to „to samo” tylko przy innym kodowaniu znaków wejścia/wyjścia; przy wejściu w formacie CPL (offset 16) wyniki będą inne.

- **`Finish` w Rust nie jest zaimplementowane (praktycznie: panic zamiast normalnego końca)**
  - Rust ma stan `Pc::Finish`, ale `finish()` jest `todo!()`.
  - Skutek: nawet jeśli logika dojdzie do „normalnego końca”, wykonanie zakończy się błędem wykonania, a nie spokojnym wyjściem jak w CPL.

- **`Monitor11`: w CPL w dostarczonym pliku brak algorytmu, Rust ma rozbudowaną procedurę**
  - Tekst CPL w tym repo urywa się na nagłówku `Monitor11`.
  - Rust implementuje konkretną logikę drukowania „Current macros…” i dalszy przepływ sterowania.
  - Skutek: zachowanie po błędach nieodwracalnych jest w praktyce inne niż wynika z dostarczonego „oryginału” (bo tam nie ma opisu).

## Tabela mapowania stanów (CPL → Rust) + różnice algorytmiczne

| Obszar | CPL (etykieta / warunek) | Rust (stan / warunek) | Różnica w algorytmie / skutek |
|---|---|---|---|
| Start – wejście | `Start: NextCh` | `Pc::Start` → `next_ch()` | Zasadniczo zgodne (krok „pobierz znak” na początku pętli). |
| Start – cytowanie | `if A = '<' do q := q+1; go to Q2` | `CH_OPEN` → `q += 1; Pc::Q2` | Zgodne. |
| Start – definicja | `A='§' → Fn` | `CH_DEF='&'` → `Pc::Fn` | Semantycznie istotne: inny znak uruchamia definicję (wejście z `§` nie zadziała bez translacji). |
| Start – separator `,` | `A=',' → NextItem` | `CH_ARGSEP` → `h==0 ? Copy : NextItem` | Rust przenosi warunek z `NextItem` do `Start`; funkcjonalnie równoważne dla `H=0`. |
| Start – aplikacja `;` | `A=';' → Apply` | `CH_APPLY` → `h==0 ? Copy : Apply` | Rust przenosi warunek z `Apply` do `Start`; funkcjonalnie równoważne dla `H=0`. |
| Start – argument `~` | `A='~' → LoadArg` (a w `LoadArg` dopiero `P=0` → `H=0→Copy` lub `Monitor2`) | `CH_LOADARG` → `p==0 ? Copy : LoadArg` | Różnica semantyczna: ścieżka do `Monitor2` zanika, bo przy `p==0` Rust nie wchodzi do `LoadArg`. |
| Start – `Marker` | `A=Marker → EndFn` | `MARKER` → `h==0 && c==0 ? Finish : EndFn` | Różnica semantyczna: Rust traktuje EOF jako „normalny koniec” w stanie bazowym. |
| Start – `>` | `A='>' → Exit` | `CH_CLOSE` → `h==0 && c==0 ? Finish : Exit` | Rust ma dodatkowe „ciche zakończenie” na `>` gdy VM „pusta”. |
| Copy | `Copy: Load` | `Pc::Copy` → `load(); Pc::Scan` | Zgodne. |
| Scan | `Scan: if q=1 go to Start` (inaczej `Q2`) | `q==1 ? Start : Q2` | Zgodne. |
| Q2 – pobranie znaku | `Q2: NextCh` | `Pc::Q2` → `next_ch()` | Zgodne. |
| Q2 – logika cytowania | (W pliku CPL fragment wygląda nietypowo) | Rust: `<` zwiększa `q`, `>` zmniejsza `q` | Różnica wg literalnego tekstu CPL; możliwy błąd transkrypcji w CPL. |
| Fn | `Fn: ...; go to Start` | `op_fn()` → `Start` | Zgodne. |
| NextItem | `if H=0 go to Copy; ...; go to Start` | `op_next_item()` analogicznie | Zgodne. |
| Apply | Sekwencja z jednoczesnymi podstawieniami, `Find[P+2]`, `JumpIfMarked`, `C:=W+1` | `op_apply()` analogicznie | Zasadniczo zgodne; Rust dodaje strażniki zakresu (błędy wewnętrzne). |
| LoadArg | `P=0` → (`H=0→Copy` lub `Monitor2`), potem wybór argumentu i kopiowanie | `op_load_arg()` analogicznie | Różnica „wejściowa”: przez bramkę w `Start` przypadek `P=0 && H!=0` nie trafi do `Monitor2`. |
| EndFn | Procedura „zwijania ramki” i kopiowania wyników | `op_end_fn()` analogicznie | Zgodne w logice; Rust ma dodatkowe kontrole zakresu. |
| Exit | `unless C=H=0 go to Monitor8; Finish` | `op_exit()` → `Pc::Finish` | Zgodne w sprawdzeniu. |
| DEF | Modyfikacja `ST[H]`, ustawienia `ST[P-1], ST[P+5], E`, potem `EndFn` | `op_def()` analogicznie | Zgodne. |
| VAL | `Find[P+6]`, kopiuj do `Marker`, potem `EndFn` | `op_val()` analogicznie | Zgodne. |
| UPDATE | `Monitor9` gdy za długi update | `op_update()` analogicznie | Zgodne. |
| BIN | (W pliku CPL) dla `'-'` wygląda jak `-W*W` | Rust: dla `'-'` zapisuje `-W` | Twarda różnica semantyczna wg tego pliku CPL (możliwa literówka w CPL, ale różnica istnieje w porównaniu 1:1). |
| DEC | Wypisywanie liczby dziesiętnej przez `Char/Quot/Rem` | Rust analogicznie, ale ASCII | Logika podobna; różni się kodowanie cyfr. |
| BAR | W CPL domyślnie `Rem[W,A]` | Rust: `Rem` tylko dla `'R'`, inaczej błąd | Różnica semantyczna dla nieznanych operatorów. |
| Monitor11 | W dostarczonym CPL brak ciała | Rust ma pełną procedurę | Nieporównywalne 1:1 z tym plikiem CPL; zachowanie po błędach jest inne. |
| Finish | Koniec programu | Rust: `todo!()` | Praktyczna różnica: brak normalnego zakończenia. |
