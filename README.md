# Problem układania planu zajęć

## Etap 1. (2-3 tygodnie)
### Parsowanie danych i podstawyw algortym jednowątkowy
Dane pobieramy ze zbiorów udostępnionych podczas [ITC 2019](https://www.itc2019.org/instances/all) (International Timetabling Competition).
Format danych opisany jest [pod tym linkiem](https://www.itc2019.org/format).

Algortym genetyczny
TODO: krótki opis tego algorytmu, w szczególności jak mierzymy fitness danego planu

## Etap 2. (1 tydzień)
### Parallelizacja na CPU
Możliwości parallelizacji:
- Obliczenie fitness jest niezależne dla każdego planu.
- W ramach jednego planu różne wątki mogą sprawdzać różne pary zajęć pod względem konfliktów. Każdy
wątek dostaje swój przydział par i sumuje karę ze wszystkich konfliktów między tymi parami. Potem
wszystkie te częściowe sumy są sumowane redukcją aby otrzymać całkowitą karę dla całego planu.

## Etap 3. (2 tygodnie)
### Przepisanie na GPU
TODO

# Etap 4. (1 tydzień)
### Testy
Jakości planu w zależności od czasu obliczeń, jak parametryzacja algortymu wpływa na wynik itp.

# Etap 5. (2-3 tygodnie)
### Inne podejścia
Testy innych metod rozwiązujących problem, niekoniecznie na GPU.
Porównanie tych innych metod do algortymu genetycznego.

# Etap 6. (do deadline'u)
### Podsumowanie
Zebranie całej pracy w jeden raport.
