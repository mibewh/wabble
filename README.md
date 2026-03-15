# Wabble

A cross-platform turn-based game platform built with Rust. The first game is a Scrabble-style word game with local multiplayer and AI opponents. Online multiplayer, web, and mobile builds are on the roadmap.

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange)
![Bevy](https://img.shields.io/badge/Bevy-0.18-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## About

Wabble is a game platform, not just a single game. It provides a generic framework for turn-based games — lobby management, player profiles, match history, networking — and individual games plug in via a trait interface. The word game is the first module, but the architecture supports adding entirely different games that reuse the same platform infrastructure.

### Word Game Features

- **Standard rules**: 15x15 board with bonus squares (DL, TL, DW, TW), 100-tile bag, 7-tile rack
- **Full scoring**: Letter values, bonus multipliers, +50 bingo bonus for using all 7 tiles
- **Local multiplayer**: Hot-seat play for 2-4 players with turn transition screens
- **AI opponents**: Three difficulty levels powered by GADDAG-based move generation
  - **Easy** — picks weaker moves for a casual game
  - **Medium** — picks from the top moves with some randomness
  - **Hard** — maximizes score with rack-leave evaluation
- **Drag-and-drop**: Click and drag tiles from your rack to the board

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2024)

### Building the Dictionary

The AI and word validation require compiled dictionary files. Build them from the included word list:

```sh
cargo run -p dict-builder -- --input data/test_words.txt --output assets/dict.fst --gaddag assets/gaddag.fst
```

### Running the Game

```sh
# With dev optimizations (recommended for development)
cargo run -p wabble-client --features dev,desktop

# Release build
cargo run -p wabble-client --release
```

### Running Tests

```sh
cargo test --workspace
```

## Project Structure

```
crates/
  wabble-platform/   Game-agnostic framework (traits, match lifecycle)
  wabble-words/      Word game logic (board, tiles, scoring, validation)
  wabble-dict/       Dictionary (FST word lookup, GADDAG for AI)
  wabble-ai/         Word game AI (move generation, difficulty levels)
  wabble-client/     Bevy desktop application
tools/
  dict-builder/      CLI to compile word lists into FST/GADDAG files
data/
  test_words.txt     Test word list (~2300 words)
assets/
  dict.fst           Compiled dictionary (generated)
  gaddag.fst         Compiled GADDAG (generated)
```

## How It Works

### Platform Layer

The core abstraction is the `TurnBasedGame` trait. Any game implementing it gets automatic support for match management, serialization, and (eventually) networking:

```rust
pub trait TurnBasedGame: Clone + Send + Sync {
    type State;        // Board, pieces, etc.
    type PlayerState;  // Hand, rack, hidden info
    type Action;       // What a player can do on their turn
    type Config;       // Game settings
    // ...
    fn apply_action(&mut self, player_idx: usize, action: Self::Action) -> Result<TurnResult, Self::Error>;
}
```

The `GameAi` trait provides a generic interface for AI opponents across any game type.

### Word Game AI

Move generation uses a [GADDAG](https://en.wikipedia.org/wiki/GADDAG) data structure — a specialized directed acyclic graph that enables efficient enumeration of all valid moves on a Scrabble board. The implementation follows the Steven Gordon algorithm with anchor detection, cross-check computation, and recursive left/right extension.

## Roadmap

### Implemented

- [x] Game framework with `TurnBasedGame` trait
- [x] Complete word game rules (board, tiles, scoring, placement validation)
- [x] FST-based dictionary and GADDAG data structures
- [x] Dictionary builder CLI tool
- [x] Bevy desktop client with board rendering and drag-and-drop
- [x] Local hot-seat multiplayer (2-4 players)
- [x] AI opponents with three difficulty levels
- [x] Turn transition screens for hot-seat play

### Coming Soon

- [ ] **Online multiplayer** — SpacetimeDB server with matchmaking, lobbies, and persistent games
- [ ] **Web build** — Play in the browser via WASM
- [ ] **Player profiles** — Stats, ELO rating, match history
- [ ] **Mobile** — iOS and Android builds with touch-optimized UI
- [ ] **Additional games** — New game modules that plug into the same platform

## License

MIT
