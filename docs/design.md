# ChessGame Design Plan

This is the living design document for the project. It records what we're
building, the decisions we've made, and the roadmap. Update it whenever a
decision changes.

New to a term? Every one used here is explained in plain language in the
[glossary](concepts/00-glossary.md).

## 1. Goals

- Build a chess engine from scratch in **Rust**, with learning as the main goal.
- Push the engine as strong as we can, one technique at a time, so that each
  optimization is understood and **measured** before moving on.
- Later, build a playable **web UI** on top of the engine.
- Document every step: what changed (`docs/devlog.md`) and the ideas behind it
  (`docs/concepts/`).

## 2. Decisions so far

| Topic | Decision | Why |
|---|---|---|
| Language | Rust | Fast like C/C++, safer, and a good language to learn on a real project |
| Platform | UCI engine first, web UI later | Lets us play and test the engine immediately in existing GUIs, then add our own UI |
| Web UI | Rust engine compiled to WebAssembly + TypeScript frontend | Runs in any browser, easy to share |
| Strength goal | As strong as possible, in phases | Each phase teaches one family of techniques |
| Documentation | `docs/devlog.md` + `docs/concepts/` in the repo | Docs grow with the code, one commit per step |

Still open: UI details (features, online play or local only). We'll decide
those when we reach the UI phase.

## 3. Architecture

A Cargo workspace with small crates, so the engine core doesn't depend on any UI:

```
ChessGame/
├── Cargo.toml          # workspace
├── engine/             # core library: board, movegen, search, eval (no I/O)
├── uci/                # binary: UCI protocol wrapper around `engine`
├── tools/              # perft runner, test-position suites, tuning scripts
├── web/                # later: wasm-bindgen wrapper + TypeScript UI
└── docs/
    ├── design.md
    ├── devlog.md
    └── concepts/
```

### 3.1 Board representation

- **Bitboards**: one `u64` per (color, piece type), plus combined occupancy
  bitboards. Bit 0 = a1, bit 63 = h8 (little-endian rank-file mapping).
- A **mailbox** array `[Option<Piece>; 64]` alongside, for fast "what's on this
  square?" lookups.
- Game state: side to move, castling rights (4 bits), en passant square,
  halfmove clock, fullmove number.
- **Zobrist hash** (a 64-bit key per position), updated incrementally on each
  move, used for repetition detection and the transposition table.
- Moves packed into a `u16` (from, to, flags for promotion, castling, en passant).
- **FEN** parsing and printing for loading and saving positions.

### 3.2 Move generation

- Precomputed attack tables for knights, kings and pawns.
- Sliding pieces (bishops, rooks, queens): start with simple loop-based
  "classical" rays, then upgrade to **magic bitboards** (with PEXT as an
  optional later experiment).
- Legal move generation using pin and check masks (no "make the move and see if
  the king is attacked" step).
- Correctness verified with **perft** (counting every leaf node to a fixed
  depth) against known results for standard test positions. This is the most
  important test suite in the whole project.

### 3.3 Search

Built up step by step (see roadmap): negamax → alpha-beta → iterative
deepening → transposition table → quiescence → move ordering → pruning and
reductions → multi-threading (Lazy SMP).

Time management: split the remaining clock time across moves, and stop
searching when the budget is spent.

### 3.4 Evaluation

Classical first: material → piece-square tables → tapered (middlegame/endgame)
eval → pawn structure, mobility, king safety. Then tuning (Texel) and finally
**NNUE**, a small neural network trained on positions from self-play.

### 3.5 Testing and measuring strength

- **Unit tests + perft** for correctness.
- **Tactical test suites** (e.g. WAC, "Win At Chess") to catch search regressions.
- **Self-play matches** with `fastchess` or `cutechess-cli`: the new version
  plays the old one, with **SPRT** (a statistical test) deciding whether a
  change actually gained Elo. Every optimization phase must show a measured gain.
- Gauntlets against known engines to estimate an absolute rating.

### 3.6 UCI

The standard text protocol used by chess GUIs: `uci`, `isready`, `position`,
`go`, `stop`, `quit`, plus `info` output during search. Supporting it means we
can play our engine in Cute Chess, Arena, or on Lichess (as a bot) for free.

### 3.7 Web UI (later phase)

- Engine compiled to WASM with `wasm-bindgen`, running in a Web Worker so
  the page stays responsive while it thinks.
- TypeScript frontend: drag-and-drop board, move list, undo, flip board,
  difficulty levels, and a PGN import/export.
- Details to be decided together when we get there.

## 4. Roadmap

Each phase ends with a working, tested engine and devlog + concept entries.
Elo numbers are rough guides, not promises.

### Phase 0: Setup
- Install Rust, create the Cargo workspace, set up CI (build, test, clippy).
- Concepts: Cargo, crates, modules, `cargo test`, debug vs release builds.

### Phase 1: The rules of chess
- Board, pieces, FEN, move encoding, make/unmake move, legal move generation.
- Passes perft for all standard test positions.
- Concepts: bitboards, attack tables, pins and checks, perft. Rust: enums,
  `match`, structs, ownership/borrowing.

### Phase 2: First playable engine (~1200 to 1500)
- UCI protocol, material + piece-square table eval, negamax with alpha-beta,
  fixed-depth search.
- Concepts: minimax, negamax, alpha-beta, piece-square tables, UCI.

### Phase 3: Searching smarter (~1800 to 2000)
- Iterative deepening, time management, Zobrist hashing + transposition table,
  quiescence search, move ordering (TT move, MVV-LVA, killers, history),
  repetition and 50-move draw detection.
- Set up self-play testing with SPRT here, so every later change is measured.
- Concepts: hashing, TT replacement, horizon effect, move ordering, SPRT.

### Phase 4: Faster move generation
- Magic bitboards for sliders, profiling and speed tuning.
- Concepts: magic numbers, profiling with `cargo flamegraph`, benchmarking.

### Phase 5: Better evaluation (~2000 to 2300)
- Tapered eval, pawn structure (passed/isolated/doubled), mobility, king
  safety, bishop pair, rooks on open files.
- Concepts: game phases, positional chess knowledge in code.

### Phase 6: Advanced search (~2400 to 2700)
- Principal variation search, null-move pruning, late move reductions,
  futility and reverse futility pruning, aspiration windows, check extensions,
  static exchange evaluation (SEE).
- Concepts: pruning vs reductions, risk/benefit of each, why measuring matters.

### Phase 7: Tuning (~+100 to 200)
- Texel tuning of eval weights using a large set of labelled positions.
- Concepts: gradient descent, logistic regression, overfitting.

### Phase 8: NNUE (~2800 to 3000+)
- Generate training data from self-play, train a small network (e.g. with
  `bullet`), efficiently-updatable inference in the engine, SIMD.
- Concepts: neural networks, incremental accumulators, quantization, SIMD.

### Phase 9: Multi-threading
- Lazy SMP: several threads searching the same tree with a shared TT.
- Concepts: threads, atomics, `Arc`, lock-free tables in Rust.

### Phase 10: Web UI
- WASM build, Web Worker, TypeScript board UI, difficulty levels.
- Concepts: WebAssembly, `wasm-bindgen`, Web Workers, frontend basics.

Phases 4 and 10 are flexible: we can start the web UI earlier if you want
something to show off sooner.

## 5. How we work

1. Pick the next step from the roadmap.
2. Write the code together, with tests.
3. Add a `docs/devlog.md` entry: what changed, which files, why, and the
   measured result (perft, speed, or Elo).
4. Add or update notes in `docs/concepts/` for anything new.
5. One commit (or PR) per step.

### Concept note standard

Concept notes are written for someone meeting the idea for the first time.
Each one covers:

1. **The problem**: what goes wrong, or is slow, without this idea.
2. **The idea in plain words**: no jargon, or jargon explained on first use.
3. **A worked example**: a concrete position, small tree or bit pattern,
   traced step by step (with ASCII diagrams where they help).
4. **How it looks in our code**: the relevant Rust, with the Rust features it
   uses explained too.
5. **Pitfalls**: common bugs and how we test for them.
6. **Measured effect**: what it did to perft speed, nodes searched, or Elo in
   our engine.
7. **Further reading**: Chess Programming Wiki pages and similar.

The [glossary](concepts/00-glossary.md) gives short definitions of every term,
and is updated whenever a new term appears.

## 6. References

- Chess Programming Wiki: https://www.chessprogramming.org
- The Rust Book: https://doc.rust-lang.org/book/
- UCI protocol specification
- Open source engines to read (after writing our own version first): Stockfish, Ethereal, Viridithas (Rust), Rustic (Rust, with a beginner-friendly book)
