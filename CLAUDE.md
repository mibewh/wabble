# Wabble

Cross-platform async turn-based game platform built with Rust, Bevy, and SpacetimeDB. Games plug into the platform via a trait-based interface. The first (and currently only) game module is a Scrabble-style word game.

## Architecture

The platform is designed so that entirely different games (e.g., a strategy game) can be added while reusing the lobby, profiles, networking, and match management layers. The separation is: **platform** (game-agnostic traits and infrastructure) vs **game modules** (game-specific logic, rendering, and AI).

```
wabble-platform    Game-agnostic: TurnBasedGame trait, GameAi trait, match lifecycle
wabble-words       Word game logic: board, tiles, scoring, placement validation
wabble-dict        Dictionary: FST-based word lookup, GADDAG for AI move generation
wabble-ai          Word game AI: GADDAG move enumeration, difficulty levels
wabble-client      Bevy app: shell UI, game-specific rendering plugins
wabble-server      (planned) SpacetimeDB server for multiplayer
```

### Key traits

- **`TurnBasedGame`** (`wabble-platform::game_trait`) — Central abstraction all games implement. Has 6 associated types: State, PlayerState, Action, PublicView, Config, Error. Any game implementing this trait automatically works with the platform's match management, serialization, and networking.
- **`GameAi<G: TurnBasedGame>`** (`wabble-platform::ai`) — Generic AI trait. Game-specific AI crates implement this for their game type.
- **`WordValidator`** (`wabble-words::game`) — Internal to the word game, not part of the platform. Allows swapping dictionary implementations or using `AcceptAllValidator` for tests.

### Crate dependency graph

```
wabble-platform  (no heavy deps, serde only)
    ↑
wabble-words  (depends on wabble-platform)
    ↑
wabble-dict   (depends on fst crate, no game deps)
    ↑
wabble-ai     (depends on wabble-words, wabble-dict, wabble-platform)
    ↑
wabble-client (depends on all above + bevy)
```

## Tech stack

- **Rust edition 2024**, workspace with resolver 2
- **Bevy 0.18** game engine for the client
- **fst** crate (v0.4) for FST-based dictionary and GADDAG storage
- **SpacetimeDB** (planned) for multiplayer server
- Dev profile: `opt-level = 1` for own code, `opt-level = 3` for deps (playable framerates during dev)
- `bevy/dynamic_linking` feature behind `dev` cargo feature for fast iteration

## Word game specifics

- Standard 15x15 board with DL/TL/DW/TW bonus squares
- 100-tile bag with standard letter distribution and point values
- Placement validation: single row/col, contiguous, connected to existing tiles, first move crosses center
- Scoring: letter values + bonus multipliers, +50 bingo bonus for using all 7 tiles
- Game end: bag empty + player empties rack, or 6 consecutive zero-score turns
- Final scoring: subtract remaining rack tile values

### GADDAG

The AI uses a GADDAG (Directed Acyclic Word Graph) encoded as an FST for move generation. Key encoding details:
- Separator byte: `>` (0x3E) divides reversed prefix from forward suffix
- For word "CARE": entries are `C>ARE`, `AC>RE`, `RAC>E`, `ERAC>`
- Move generation uses the Steven Gordon algorithm: anchor detection, cross-check computation, recursive left/right extension
- The GADDAG's first letter from root IS the anchor letter (not a letter to its left)

### Dictionary files

- `data/test_words.txt` — ~2300 word test list
- `assets/dict.fst` — Compiled FST dictionary (built by `tools/dict-builder`)
- `assets/gaddag.fst` — Compiled GADDAG (built by `tools/dict-builder`)
- Build with: `cargo run -p dict-builder -- --input data/test_words.txt --output assets/dict.fst --gaddag assets/gaddag.fst`

### AI difficulty

- **Easy**: picks from the bottom third of generated moves
- **Medium**: picks uniformly from the top 5 moves
- **Hard**: maximizes score + rack-leave evaluation (vowel/consonant balance, letter flexibility, duplicate penalty)

## Client architecture (Bevy)

### App states

`MainMenu → InGame → GameOver` (defined in `app_states.rs`)

### Plugins

- **ShellPlugin** — Camera setup (generic, reusable)
- **LobbyPlugin** — Main menu with hot-seat (2-4 players) and vs-AI (Easy/Medium/Hard) options
- **WordsGamePlugin** — Board rendering, rack display, drag-and-drop input, HUD, AI turn system
- **NetworkPlugin** — Placeholder for SpacetimeDB (Phase 5-6)
- **ProfilePlugin** — Placeholder for player stats
- **AudioPlugin** — Placeholder for sound effects

### Key resources

- `ActiveMatch` — Holds the `WordGame` instance and player count
- `PendingPlacement` — Tiles placed on board this turn but not yet submitted
- `DragState` / `DragInfo` — Drag-and-drop state for rack tiles
- `AiOpponent` — Wraps `Arc<WordGameAi>` with the AI's player index
- `AiMoveTimer` — 0.8s delay before AI moves for UX
- `TurnTransition` — Overlay between hot-seat turns to hide opponent's rack
- `StatusMessage` — Last action result shown in the HUD
- `SelectedRackTile` — Legacy click-to-select (superseded by drag-and-drop)

### Input model

Tiles are placed via drag-and-drop: click a rack tile to start dragging, release over a board cell to place it. A ghost sprite follows the cursor during drag. Clicking a pending tile on the board recalls it to the rack. The "Recall" button returns all pending tiles.

## Build & run

```sh
# Run the client (dev mode with dynamic linking)
cargo run -p wabble-client --features dev,desktop

# Build the dictionary files from a word list
cargo run -p dict-builder -- --input data/test_words.txt --output assets/dict.fst --gaddag assets/gaddag.fst

# Run all tests
cargo test --workspace

# Run clippy
cargo clippy --workspace
```

## Current status

**Keep this section up to date as work progresses.** When a phase is completed or new work is done, update this section to reflect the current state. Also update the **Roadmap** section in `README.md` to keep the public-facing status in sync (move items from "Coming Soon" to "Implemented", add new items as needed).

### Completed phases

1. **Platform traits + word game core logic** — `wabble-platform` and `wabble-words` fully implemented. `TurnBasedGame` trait, full board/tile/scoring/placement logic, game end conditions.
2. **Dictionary (FST + GADDAG)** — `wabble-dict` with `FstDictionary` and `Gaddag`. `tools/dict-builder` CLI builds both from word lists. ENABLE-based test word list at `data/test_words.txt`.
3. **Bevy client with local hot-seat play** — `wabble-client` with board rendering, drag-and-drop tile placement, HUD, turn transitions for 2-4 player hot-seat.
4. **AI opponents (GADDAG move generation)** — `wabble-ai` with GADDAG-based move enumeration, three difficulty levels (Easy/Medium/Hard), integrated into client with menu options.

### Current state

- All phases 1-4 compile cleanly, pass clippy, pass tests
- The game is playable locally: hot-seat multiplayer and vs AI all work
- No server or networking code exists yet

### Next up

5. **SpacetimeDB server** (`wabble-server`) — Server-authoritative multiplayer with generic tables (Player, Match, Turn, Lobby) and game-specific validation dispatch
6. **Multiplayer client integration** — SpacetimeDB Rust SDK in `NetworkPlugin`, lobby UI, real-time game sync
7. **Web (WASM) build** — `wasm32-unknown-unknown` via trunk, HTTP-fetched dictionary
8. **Mobile + polish + additional games** — iOS/Android, touch UI, second game module

## Design decisions

- **Visual style**: Simple colored rectangles for now (easy to replace with proper art later)
- **Auth**: SpacetimeDB anonymous tokens initially, upgrade to OIDC later if needed
- **WordValidator orphan rule**: `WordValidator` lives in `wabble-words::game`, not in `wabble-platform`. `FstDictionary` implements it in `wabble-dict`. The AI crate (`wabble-ai`) uses `FstDictionary::contains()` directly for cross-check validation rather than going through the trait, to avoid orphan rule issues.
- **AI fallback**: If the AI's chosen move is rejected by placement validation, it falls back to Pass to prevent infinite retry loops.
- **Game end condition**: 6 consecutive zero-score turns (not just passes — a play that scores 0 also counts), or bag empty + a player empties their rack.
