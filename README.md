# Problem układania planu zajęć

## Etap 1. (2-3 tygodnie)
### Parsowanie danych i podstawowy algortym jednowątkowy
Dane pobieramy ze zbiorów udostępnionych podczas [ITC 2019](https://www.itc2019.org/instances/all) (International Timetabling Competition).
Format danych opisany jest [pod tym linkiem](https://www.itc2019.org/format) ([przykładowy input](data/itc2019/sample.xml)).

### Algorytm genetyczny
Zarys podstawowoych kroków algorytmu:
```
population = init_population(SIZE) // inicjalizacja losowa, populacja = zbiór planów
for generations in 0..N_GENERATIONS {
    evaluate_population_fitness(...) // obliczamy fitness dla każdego planu
    selection(...) // wybór jednostek, które zostaną ze sobą "skrzyżowane"
    crossover(...) // tworzymy nową populację ze skrzyżowania wybranych wcześniej jednostek między sobą
    apply_mutations(...) // losowe zmiany w planach aby "odblokować" więcej potencjalnych ścieżek ewolucji
}
```

### Fitness
Dla każdego planu można policzyć *karę* na którą składają się:

Tzw. *hard constraints* - bez spełnienia tych ograniczeń plan uznaje się za
nieprawidłowy. W praktyce w takich przypadkach po prostu dodajemy do kary dużą wartość.
Przykładowo, dwa zajęcia nie mogą jednocześnie używać tej samej sali

Tzw. *soft constraints* - za ich złamanie obowiązuje kara. W itc2019 każdy problem
ma osobne wagi dla różnych kategorii kar, przykładowo:
```xml
<optimization time="2" room="1" student="2" distribution="1"/>
```

- `time` - każde z zajęć ma wyznaczone godziny, w których mogą się odbywać, niektóre
z nich mają niezerową karę.
- `room` - podobnie jak `time`, tylko dla sal.
- `student` - kara, kiedy nakładają się zajęcia dla studenta.
- `distribution` - przykładowo zajęcia A i B powinny odbyć się tego samego dnia.
Dokładniej opisane [tutaj](https://www.itc2019.org/format#distributions).

## Etap 2. (1 tydzień)
### Parallelizacja na CPU
Możliwości parallelizacji:
- Obliczenie fitness jest niezależne dla każdego planu.
- W ramach jednego planu różne wątki mogą sprawdzać różne pary zajęć pod względem konfliktów. Każdy
wątek dostaje swój przydział par i sumuje karę ze wszystkich konfliktów między tymi parami. Potem
wszystkie te częściowe sumy są sumowane redukcją aby otrzymać całkowitą karę dla całego planu.

## Etap 3. (2 tygodnie)
### Przepisanie na GPU
- Optymalna dla GPU reprezentacja danych
TODO

## Etap 4. (1 tydzień)
### Testy
Jakości planu w zależności od czasu obliczeń, jak parametryzacja algortymu wpływa na wynik itp.
- Porównanie z innymi algorytmami z ITC2019.
- Czy poprawa w jakości planu jest warta większej mocy obliczeniowej/czasu obliczeń?

## Etap 5. (2-3 tygodnie)
### Inne podejścia
Testy innych metod rozwiązujących problem, niekoniecznie na GPU.
Porównanie tych innych metod do algortymu genetycznego.
### Problem dokonania małych zmian w danym planie
- Algorytm powinien działać real-time.
- Dodatkowa kara przy fitness za każdą zmianę.

## Etap 6. (do deadline'u)
### Podsumowanie
- Zebranie całej pracy w jeden raport.
- Prezentacja?
- Wpis na blogu?
