# sloth

Basic chess engine in Rust. Slothful as it uses existing packages where it can. 

All the chess logic is provided by the excellent [cozy-chess](https://github.com/analog-hors/cozy-chess)

At present it consists of only 3 files. The **main.rs** includes a simple handling of the [UCI protocol](https://www.chessprogramming.org/UCI).
The **evaluation.rs** is largely based on code included in the excelent engine [akimbo](https://github.com/jw1912/akimbo). This just gets the evaluation from NNUE data originally created for [Leela Chess Zero](https://lczero.org/).

The final file is **search.rs** which is very much a WIP. It has the most basic functionality but does include:

    * Negamax search
    * Transposition table
    * Quiescence Search

The engine already performs decently.

## Plans

To add enough to the search to make it perform well. Features will only be added if they make a large differnce as the intention is to keep this as simple as possible.

## Purpose

To create an engine with decent strength but minimal coding.

To provide this as a base for new independent engines where the code can be easily replaced and extended.

To allow easy swapping out of functionality - e.g. using NNUE data from other sources.