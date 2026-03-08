# Problem układania planu zajęć

Wejście: plik XML definiujący zbiór sal zajęciowych (wraz z ich pojemnościami i harmonogramem dostępności), zbiór zajęć (wraz z ich wymaganiami np. co do możliwych slotów czasowych lub sal, w
których mogą się odbywać), zbiór studentów (każdy student posiada listę zajęć, na które jest zapisany) i dodatkowe ograniczenia.

Wyjście: zbiór zajęć, każde z nich ma zdefiniowane tygodnie, dni, sloty czasowe i sale, w których będzie się odbywać oraz zbiór studentów.

## 1. Parsowanie danych
Dane pobieramy ze zbiorów udostępnionych podczas [ITC 2019](https://www.itc2019.org/instances/all) (International Timetabling Competition).

Format danych opisany jest [pod tym linkiem](https://www.itc2019.org/format).

## 2. Algortym na CPU (zarys)
Implementujemy algortym genetyczny. Podstawową jednostką w takim algorytmie jest *chromosom* -- w tym przypadku będzie to $n$ elementowy wektor
$[p_1, p_2, \dots, p_n]$, gdzie $p_i = (r_i, t_i)$ oznacza, że $i$-te zajęcia ustalamy w $r_i$-tej sali (z listy sal dozwolonych dla tych zajęć) i w $t_i$-tej konfiguracji czasowej (spośród
listy dozwolonych konfiguracji).

Algortym inicjujemy pewną liczbą losowych chromosomów, następnie wykonujemy kolejne iteracje:
1. Oblicz "fitness" każdego chromosomu (ważona suma kar za konflikty).
2. Topowe kilka procent chromosomów "krzyżujemy" ze sobą (np. wymieniamy połowę wartości z wektora między dwoma), w wyniku czego powstaje nowa populacja.
3. Aplikujemy mutacje, np. losowo (z niewielkim prawdobodobieństwem) zmieniamy niektóre pary w chromosomach.

Po pewnej liczbie iteracji (lub gdy "fitness" przestaje się polepszać) zatrzymujemy algorytm.

## 3. Możliwości parallelizacji
Obliczenie "fitnessu" jest niezależne dla każdego chromosomu, podobnie jak mutacje. Być może da się też przyspieszyć krok krzyżowania.

## 4. Eksperymenty
- Jak zmiana parametrów algortymu wpływa na jakość wyniku i czas działania?
- Czy można szybko wygenerować nowy plan na podstawie starego, jeżeli różnica pomiędzy ograniczeniami jest niewielka?
