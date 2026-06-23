# Coloração de Grafos

Trabalho final da disciplina de **Projeto e Análise de Algoritmos** do **Mestrado
Acadêmico em Computação Aplicada** da **Universidade do Estado de Santa Catarina
(UDESC)**.

Este projeto implementa e compara algoritmos para o problema NP-Completo de
`k`-coloração de grafos.

## Como executar

```sh
cargo test
cargo run --bin coloring -- --algorithm dsatur --n 12 --k 3 --density 0.35 --seed 1
cargo run --bin coloring -- --algorithm bruteforce --n 10 --k 3 --density 0.35 --seed 1
cargo run --bin coloring -- --algorithm welsh_powell --n 20 --k 4 --density 0.25 --seed 1
cargo run --release --bin benchmark
```

## Algoritmos

- `bruteforce`: enumera todas as `k^n` atribuições e valida cada coloração no final.
- `welsh_powell`: heurística gulosa clássica por grau decrescente.
- `dsatur`: backtracking exato com escolha por grau de saturação, desempate por grau e bitsets.

O benchmark reporta chamadas recursivas, atribuições completas, repetições e tempo
médio. O DSATUR/backtracking reduz o espaço de busca observado, mas o pior caso
permanece exponencial.

## Artigo

O diretório `article/` contém o artigo em formato SBC (`article.tex`), que
apresenta a fundamentação teórica do problema de coloração de grafos, descreve a
implementação dos algoritmos em Rust, compara os resultados experimentais
obtidos com o benchmark e tira conclusões sobre o trade-off entre exatidão,
eficiência e escalabilidade das abordagens estudadas.

O PDF compilado (`article/article.pdf`) também está incluído no repositório para
consulta direta, sem necessidade de recompilar.

Para regenerar o PDF:

```sh
cd article
latexmk -pdf -shell-escape article.tex
```
