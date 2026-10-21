# Glossary

Every term used in [the design plan](../design.md), in plain language. These
are short definitions so you can follow the plan. Each important one gets its
own detailed concept note (with examples, diagrams and code) when we reach
the phase that uses it. The phase is shown in brackets.

Terms are grouped by topic, roughly in the order you'll meet them.

---

## Chess basics

**FEN (Forsyth-Edwards Notation)** [Phase 1]
A one-line text format that describes a full position. The starting position is:
`rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`.
The first part lists pieces rank by rank from rank 8 down to rank 1
(uppercase = White, lowercase = Black, digits = that many empty squares). Then
come side to move (`w`), castling rights (`KQkq`), en passant square (`-`),
halfmove clock (`0`) and fullmove number (`1`). Engines use FEN to load test
positions.

**PGN (Portable Game Notation)** [Phase 10]
A text format for a whole game: header tags (players, date, result) plus the
moves, e.g. `1. e4 e5 2. Nf3 Nc6`. Used for saving and sharing games.

**Castling rights**
Whether each side may still castle kingside or queenside. A right is lost
forever once the king or that rook moves, so the engine must track it (4 yes/no
flags).

**En passant**
A special pawn capture: if a pawn moves two squares and lands beside an enemy
pawn, that enemy pawn may capture it as if it had moved only one square, but
only on the very next move. The engine stores the "en passant square" for one
move.

**Halfmove clock / 50-move rule**
The number of moves since the last capture or pawn move. When it reaches 100
half-moves (50 per side), either side can claim a draw. The engine must know
this to avoid playing for a win in a position that is legally a draw.

**Threefold repetition**
If the same position occurs three times, it's a draw. Engines detect this using
the Zobrist hash (see below).

**Ply**
One move by one side. "Depth 6" means looking 6 plies ahead: 3 moves for each
side. Chess players say "move" for a pair of plies, which is confusing, so
engine programmers say "ply".

**Pin**
A piece that can't move (or can only move along one line) because moving it
would expose its own king to attack. Example: a white knight on e2 between the
white king on e1 and a black rook on e8 is pinned.

**Check / double check**
The king is attacked. In double check, two pieces attack the king at once, so
only a king move can escape. Move generation must handle both cases.

**Zugzwang**
A position where any move makes things worse, so you'd rather "pass" but
can't. Matters for null-move pruning (below), which assumes passing is never
good.

---

## Board representation

**Bitboard** [Phase 1]
A 64-bit integer (`u64` in Rust) where each bit stands for one square. One
bitboard can answer "which squares have white pawns?" With 12 bitboards (6
piece types × 2 colors) you have the whole board. Bit operations then work
on all 64 squares at once: `white_pawns << 8` moves every white pawn up one
rank in a single CPU instruction.

**Square mapping (little-endian rank-file)** [Phase 1]
The rule for which bit means which square. We use bit 0 = a1, bit 1 = b1, ...,
bit 7 = h1, bit 8 = a2, ..., bit 63 = h8. So `square = rank * 8 + file`.

**Occupancy**
A bitboard of all occupied squares (or all squares occupied by one color).
Made by OR-ing the piece bitboards together. Used to know where sliding
pieces get blocked.

**Mailbox**
A plain array of 64 entries, each saying which piece (if any) is on that
square. Bitboards are fast for "where are all the knights?", and a mailbox is
fast for "what is on e4?". We keep both in sync.

**Move encoding** [Phase 1]
Packing a move into a small number. A `u16` has 16 bits: 6 bits for the from
square (0 to 63), 6 bits for the to square, and 4 bits of flags (promotion
piece, castling, en passant, capture). Small moves mean fast copying and
compact tables.

**Make / unmake move** [Phase 1]
"Make" applies a move to the board, and "unmake" reverses it exactly. The
search makes and unmakes millions of moves per second, so this must be fast
and perfectly correct.

**Zobrist hashing** [Phase 3]
A way to give every position a (nearly) unique 64-bit number. At startup we
create a random number for each (piece, square) pair, plus some for side to
move, castling and en passant. A position's hash is the XOR of the numbers for
everything in it. Because XOR undoes itself, a move updates the hash cheaply:
XOR out the piece on its old square, XOR it in on the new one.

---

## Move generation

**Move generation (movegen)**
Code that lists every legal move in a position.

**Pseudo-legal vs legal moves**
Pseudo-legal moves follow how pieces move but might leave your own king in
check. Legal moves are the truly allowed ones. Some engines generate
pseudo-legal moves and then check each; we'll generate legal moves directly.

**Attack tables** [Phase 1]
Precomputed lookup arrays, e.g. `KNIGHT_ATTACKS[sq]` gives a bitboard of every
square a knight on `sq` attacks. Knights, kings and pawns always attack the
same squares from a given square, so we compute these once at startup.

**Sliding pieces**
Bishops, rooks and queens: they move any distance along a line until blocked.
Their attacks depend on which squares are occupied, so a simple table doesn't
work.

**Classical rays** [Phase 1]
The simple way to compute slider attacks: walk square by square in each
direction until you hit a piece or the edge. Easy to understand but slower.

**Magic bitboards** [Phase 4]
The fast way to compute slider attacks. Take the occupied squares along a
rook's or bishop's lines, multiply by a carefully chosen "magic" number, and
shift. The result is an index into a precomputed table that holds the attack
bitboard. It turns a loop into a multiply, a shift and a table lookup. The
magic numbers are found by trial and error at startup or ahead of time.

**PEXT** [Phase 4, optional]
A CPU instruction (on newer Intel/AMD chips) that extracts selected bits into
a compact number. It can replace the magic multiply for an even simpler slider
lookup.

**Pin mask / check mask** [Phase 1]
Bitboards that describe which squares pieces are allowed to move to because of
pins or checks. E.g. when in check from a rook, the check mask is the squares
between the rook and king plus the rook itself: any non-king move must land on
one of those squares.

**Perft (performance test)** [Phase 1]
Count every possible position reachable in exactly N plies. From the starting
position, depth 1 = 20, depth 2 = 400, depth 3 = 8,902, depth 5 = 4,865,609.
These numbers are known, so if ours differ, movegen has a bug. It's the
single most important correctness test for an engine.

---

## Search

**Search tree / node**
Each position reached during the search is a node, and each move is a branch to
a child node. The "leaves" are the positions at the bottom where we evaluate.

**Minimax** [Phase 2]
The basic idea of game search: I pick the move that maximizes my score, and
assume my opponent picks the reply that minimizes it, all the way down the
tree.

**Negamax** [Phase 2]
The same as minimax but written as one function. Because a position worth +3
for me is worth -3 for my opponent, we always maximize and flip the sign each
ply: `score = -search(child)`.

**Alpha-beta pruning** [Phase 2]
An optimization of minimax that gives the same answer while skipping branches
that can't matter. `alpha` is the best score I'm already guaranteed, and
`beta` is the best my opponent will allow. If a move turns out so good for me
that the opponent would never allow it (score ≥ beta), we stop looking at the
rest of that node's moves (a "beta cutoff"). With good move ordering it lets
you search about twice as deep in the same time.

**Iterative deepening** [Phase 3]
Search depth 1, then depth 2, then depth 3, and so on until time runs out.
That sounds wasteful, but each search is much faster thanks to what earlier
iterations learned (best moves go first), and you always have a finished
answer ready when time is up.

**Time management** [Phase 3]
Deciding how long to think per move, based on the remaining clock time and
increment.

**Transposition table (TT)** [Phase 3]
A big hash table (indexed by Zobrist hash) that remembers results for
positions already searched: best move, score, depth. The same position is
often reached by different move orders ("transpositions"), so this saves huge
amounts of work. Its best move is also the best guess to try first.

**Replacement scheme**
The TT has a fixed size, so when two positions map to the same slot we need a
rule for which to keep (e.g. prefer the deeper search, or the newer one).

**Horizon effect** [Phase 3]
An engine that stops searching at a fixed depth can't see what happens next.
E.g. it captures a pawn with its queen at the last ply, not seeing the queen
gets taken right after.

**Quiescence search** [Phase 3]
The fix for the horizon effect. At the end of the normal search, keep
searching only captures (and maybe checks) until the position is "quiet",
and only then evaluate.

**Move ordering** [Phase 3]
Trying the best moves first so alpha-beta cuts off as early as possible.
Usual order: TT move, then good captures, then killer moves, then other moves
by history score.

**MVV-LVA (Most Valuable Victim, Least Valuable Attacker)** [Phase 3]
A simple way to order captures: prefer capturing big pieces with small ones.
Pawn takes queen goes first, and queen takes pawn goes late.

**Killer moves** [Phase 3]
Quiet (non-capture) moves that caused a beta cutoff at the same depth in a
sibling branch. They're likely to be good here too, so we try them early.

**History heuristic** [Phase 3]
A table scoring quiet moves (by from/to square or piece/to square) by how
often they caused cutoffs anywhere in the search. Used to order the remaining
quiet moves.

**Principal variation (PV)**
The sequence of best moves the engine expects for both sides, i.e. the "main
line". Shown in the engine's output.

**PVS (Principal Variation Search)** [Phase 6]
Assumes the first move is the best. Every other move is searched with a cheap
"null window" just to prove it's worse; only if that proof fails do we
re-search it fully. Pays off because move ordering is usually right.

**Null-move pruning** [Phase 6]
"If I skip my turn and I'm still winning by a lot, this position is so good
I don't need to search it fully." We let the opponent move twice with a
shallower search, and if we still beat beta, we cut off. Must be turned off in
zugzwang-prone positions (e.g. pawn endgames).

**Late move reductions (LMR)** [Phase 6]
Moves late in the ordering are probably bad, so search them less deeply. If
one surprises us with a good score, re-search it at full depth. One of the
biggest strength gains in modern engines.

**Futility pruning / reverse futility pruning** [Phase 6]
Near the leaves, if the position is so bad that even winning a piece wouldn't
lift the score up to alpha, skip quiet moves (futility). If it's so good that
even losing a piece would keep it above beta, cut off right away (reverse
futility).

**Aspiration windows** [Phase 6]
Instead of searching with alpha = -∞ and beta = +∞, search with a narrow window
around the previous iteration's score. Narrow windows mean more cutoffs. If
the result falls outside, widen the window and search again.

**Check extensions** [Phase 6]
When a move gives check, search it one ply deeper, because checks are forcing
and often lead to tactics.

**SEE (Static Exchange Evaluation)** [Phase 6]
Works out whether a capture wins or loses material once all the recaptures on
that square are played out, without actually searching. Used to order captures
and to prune bad ones.

**Lazy SMP** [Phase 9]
The simplest way to use multiple CPU cores: run several searches at once that
share one transposition table. They naturally help each other through the
shared table.

---

## Evaluation

**Evaluation (eval)**
A function that scores a position without searching, from the side to move's
point of view.

**Centipawn (cp)**
The unit of evaluation: 1/100 of a pawn. +150 cp means "about a pawn and a
half better".

**Material**
Counting pieces: pawn 100, knight about 320, bishop about 330, rook 500, queen
900 (in centipawns). The king has no material value since it can't be lost.

**Piece-square tables (PST)** [Phase 2]
For each piece type, a table of 64 bonuses/penalties, one per square. E.g.
knights get +20 in the centre and -50 in the corners. A simple way to add
positional knowledge.

**Game phase / tapered eval** [Phase 5]
Pieces are worth different amounts in the middlegame and endgame (a king should
hide in the middlegame but be active in the endgame). Tapered eval computes a
middlegame score and an endgame score, then blends them based on how much
material is left.

**Pawn structure** [Phase 5]
- **Passed pawn**: no enemy pawns can stop it from promoting. Very valuable.
- **Isolated pawn**: no friendly pawns on neighbouring files to protect it. Weak.
- **Doubled pawns**: two of your pawns on the same file. Usually weak.

**Mobility** [Phase 5]
How many squares a piece can move to. More mobile pieces are usually better.

**King safety** [Phase 5]
How exposed the king is: pawn shield in front of it, enemy pieces attacking
nearby squares, open files near it.

**Bishop pair**
Having both bishops is worth a bonus (about half a pawn), especially in open
positions.

**Open / semi-open file**
A file with no pawns (open), or no pawns of your own (semi-open). Rooks love
them.

---

## Tuning and testing

**Elo** [all phases]
A rating number for strength. Only the difference matters: +200 Elo scores
about 75% against the weaker player. See
[How chess engines work](01-how-chess-engines-work.md).

**Self-play testing** [Phase 3]
Playing thousands of fast games between the new and old version of our engine
to see whether a change helped.

**SPRT (Sequential Probability Ratio Test)** [Phase 3]
A statistical test that keeps playing games until it's confident whether a
change gains Elo (pass) or not (fail). It stops early when the answer is
clear, saving time. Without it, small gains are hidden by the noise of random
game results.

**fastchess / cutechess-cli** [Phase 3]
Command-line tools that run engine-vs-engine matches and can apply SPRT.

**Gauntlet**
Playing our engine against several other engines of known rating to estimate
its absolute Elo.

**Tactical test suite (e.g. WAC, "Win At Chess")**
A set of positions with a known winning move. We count how many the engine
solves in a fixed time to catch search bugs.

**Texel tuning** [Phase 7]
A way to automatically tune eval weights (piece values, PST entries, bonuses).
Take a large set of positions labelled with the game result (win/draw/loss),
then adjust the weights so the eval predicts those results as well as
possible.

**Logistic regression / sigmoid** [Phase 7]
The maths behind Texel tuning: the sigmoid function turns an eval in
centipawns into a predicted win probability between 0 and 1.

**Gradient descent** [Phase 7]
An algorithm for finding the best weights: compute in which direction each
weight should move to reduce the prediction error, take a small step, and
repeat.

**Overfitting** [Phase 7]
When tuned weights fit the training positions very well but play worse in real
games. Avoided with large, varied datasets and by testing in real games.

**NNUE (Efficiently Updatable Neural Network)** [Phase 8]
A small neural network used as the evaluation function. It learns what
positions are good from millions of examples instead of hand-written rules.
"Efficiently updatable" means that after a move, only the part of the network
affected by the moved pieces is recomputed, which makes it fast enough for
search. All top engines use it now.

**Accumulator** [Phase 8]
The NNUE's first layer output, kept up to date incrementally as moves are made
and unmade. That incremental update is the "efficiently updatable" part.

**Quantization** [Phase 8]
Storing network weights as small integers (e.g. `i16`) instead of floats, so
inference is faster.

**SIMD (Single Instruction, Multiple Data)** [Phase 8]
CPU instructions that do the same operation on many numbers at once (e.g. add
16 pairs of `i16` in one instruction). Makes NNUE inference fast.

**bullet** [Phase 8]
An open-source trainer for NNUE networks, written in Rust.

---

## Protocols and tools

**UCI (Universal Chess Interface)** [Phase 2]
The standard text protocol between a chess engine and a GUI. The GUI sends
commands like `position startpos moves e2e4` and `go wtime 60000`, and the
engine replies with `info depth 10 score cp 35 pv e7e5 ...` and finally
`bestmove e7e5`. Any engine that speaks UCI can be used in any UCI GUI.

**Chess GUI (Cute Chess, Arena)**
Free desktop programs that draw a board and let you play against, or watch,
any UCI engine. We can use them before building our own UI.

**Lichess bot**
Lichess lets you connect a UCI engine to a bot account so anyone can play it
online.

---

## Rust and tooling

**Cargo** [Phase 0]
Rust's build tool and package manager: `cargo build`, `cargo test`, `cargo run`.

**Crate**
A Rust package, either a library or a program (binary). Like a library or
module in other languages.

**Workspace** [Phase 0]
Several related crates in one repo, sharing one build. Ours: `engine`
(library), `uci` (binary), `tools`, and later `web`.

**Module**
A file or block of Rust code with its own namespace (`mod board;`). Like a
`.h`/`.c` pair in C, but without headers.

**Debug vs release build** [Phase 0]
`cargo build` makes a debug build: slow, with extra checks. `cargo build
--release` turns on optimizations, and for an engine it can be 10 to 50 times
faster. Always measure speed in release mode.

**Clippy** [Phase 0]
Rust's linter: it suggests more idiomatic or safer code.

**CI (Continuous Integration)** [Phase 0]
GitHub automatically builds and tests the code on every push or PR, so broken
changes are caught right away.

**Enums and `match`** [Phase 1]
Rust enums can represent "one of several kinds", e.g. `enum Piece { Pawn,
Knight, ... }`, and `match` handles each case. The compiler checks you didn't
forget a case.

**Ownership and borrowing** [Phase 1]
Rust's core rule: each value has one owner. Other code can borrow it, either
read-only (`&board`, many at once) or mutably (`&mut board`, only one at a
time). This prevents whole classes of bugs C and C++ allow, and it's what the
compiler is checking when it complains.

**`Arc` and atomics** [Phase 9]
Tools for sharing data safely between threads. `Arc` is a shared pointer that
counts references, and atomics are integers that many threads can update
safely without locks.

**Profiling / flamegraph** [Phase 4]
Measuring where the program spends its time. A flamegraph is a picture of
that, showing which functions are the hot spots.

---

## Web

**WebAssembly (WASM)** [Phase 10]
A compact binary format that browsers can run at near-native speed. Rust can
compile to it, so our engine can run inside a web page.

**wasm-bindgen** [Phase 10]
A Rust tool that generates the glue code so JavaScript/TypeScript can call
Rust functions compiled to WASM.

**Web Worker** [Phase 10]
A background thread in the browser. Running the engine in one keeps the page
responsive while it thinks.

**TypeScript**
JavaScript with types. We'll use it for the web UI.
