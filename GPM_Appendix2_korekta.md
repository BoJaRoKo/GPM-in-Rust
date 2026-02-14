# GPM Appendix 2 (CPL) — uwagi do korekty po OCR

Poniższe punkty wskazują miejsca w pliku `GPM_Appendix2_Strachey_CPL.txt`, które wyglądają na przekłamania po OCR (typowe zamiany `l/1`, przekręcone identyfikatory, złe cudzysłowy, utracone znaki). Przy każdym punkcie podaję krótko, *co prawdopodobnie miało być*.

> Uwaga: to jest lista diagnostyczna. Nie wprowadzałem żadnych zmian w źródłowym `.txt`.

## 1) Powtórzone / przekręcone nazwy zmiennych i tablic

- **Deklaracje rejestrów**: `and H, P, P, C all = 0`
  - Prawdopodobnie: `and H, P, F, C all = 0` (w całym programie występuje `F`, a tutaj go brakuje).
- **Kopiowanie MST do ST**: `ST[k] := MSP[k]`
  - Prawdopodobnie: `ST[k] := MST[k]` (literówka: `P` zamiast `T`).

## 2) Zamiany „1” ↔ „l” (małe L) w indeksach i stałych

To najczęstszy typ błędu OCR w tym pliku — i realnie zmienia algorytm.

- **NextCh**: `C+l` zamiast `C+1`.
- **Fn / VAL**: `ST[S+l]`, `ST[W+l]` zamiast `ST[S+1]`, `ST[W+1]`.
- **EndFn**: liczne wystąpienia `P-l`, `ST[P-l]`, `ST[P+l]` itd. — praktycznie na pewno powinno być `P-1`, `ST[P-1]`, `ST[P+1]`.
- **BAR**: `ST[P+1l]` wygląda jak `ST[P+11]` (OCR zrobił `1l`).
- **DEC / Monitor11**: mieszanie `Wl` i `W1`, oraz `Wl/l0` zamiast `W1/10`.

## 3) Złe znaki interpunkcyjne / myślniki (minus)

- Pojawia się `—` (długi myślnik) zamiast `-` (minus), np. `—2**20`, `x — 16`.
- W warunkach porównań też widać tę zamianę, np. `W>P—1` zamiast `W>P-1`.

## 4) Błędne / pomieszane cudzysłowy i znaki specjalne

- W stałych znakowych i tablicy `MST` jest mieszanka `'` oraz `‘ ’`, miejscami z dziwnymi odstępami/tabulatorami.
  - Typowe po OCR; może nie psuć algorytmu, ale utrudnia jednoznaczne odczytanie.

## 5) Oczywiste literówki w nazwach etykiet/identyfikatorów

- **`Nextltem`** (małe `l`) zamiast `NextItem` (duże `I`).
- **`Sr[P+7]`** zamiast `ST[P+7]` w makrze `BIN`.
- **`MonitorlO` / `Monitorl1`** (litera `l`) zamiast `Monitor10` / `Monitor11`.
- Dodatkowe spacje w nazwach etykiet: `Monitor 10`, `Monitor 11`.

## 6) Fragmenty wyglądające na uszkodzone logicznie (nie tylko typografia)

- **Load**: `or do ST[S], 5 := A, P+1`
  - `5` wygląda jak błędnie rozpoznane `S` albo `S+1`.
  - Sama para przypisań sugeruje, że miało tu być klasyczne: `ST[S] := A` oraz `S := S+1` (albo równoważne), a nie `P+1`.
- **Q2**: blok cytowania wygląda niespójnie (np. warunek `if A ≠ '>' go to Copy` i późniejsze `q := q-1`), jakby OCR poprzestawiał znaki/warunki.
- **DEC**: `repeat until 1+1 <1`
  - To niemal na pewno śmieć po OCR. Sensowny warunek zakończenia jest zwykle typu `repeat until W1 < 1` (lub równoważny).
- **BAR**: w liście operatorów wygląda na gubienie przecinków/łamań (np. po `W*A`).
- **Item**: `ST[x+k:]` oraz `Write['...*t{Incomplete)']`
  - Dwukropek `:` i `{` wyglądają jak artefakty OCR; spodziewane raczej `ST[x+k]` i `...(Incomplete)`.
- **Monitory**: w tekstach monitorów widać literówki typu `off` zamiast `of` oraz gubienie sekwencji `*n` (np. `result is n` zamiast `result is *n`).

## 7) Monitor11 — mieszanie `W1` i `Wl`

- W `Monitor11` powtarza się naprzemiennie `W1`/`Wl` oraz konstrukcje typu `Wl := W1+ST[W1]`.
  - To klasyczny błąd OCR: `1` ↔ `l`. W takim kodzie to krytyczne, bo z jednej zmiennej robią się „dwie”.
