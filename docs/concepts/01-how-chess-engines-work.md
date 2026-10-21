# How Chess Engines Work

Used in: Planning (overview of the whole project)

## Elo: measuring strength

Elo is a rating number for playing strength. It's relative: what matters is
the gap between two players.

| Rating | Who |
|---|---|
| 800 to 1000 | Beginners |
| ~1500 | Casual club players |
| ~2000 | Strong club players |
| 2500+ | Grandmasters |
| ~3500 | Top engines (Stockfish) |

A 200-point gap means the stronger side scores about 75%. A 400-point gap is
about 90%. We'll use Elo to measure whether each change actually made our
engine stronger, by playing the new version against the old one.

## The three jobs of an engine

### 1. Know the rules (move generation)

Given a position, list every legal move. Simple in principle, but castling, en
passant, promotion, and "you can't leave your own king in check" make it the
most bug-prone part of any engine.

**Bitboards** are a fast way to store the board. A chessboard has 64 squares
and a `u64` has 64 bits, so one number can say "which squares have a white
knight": one bit per square. Moves and attacks are then computed with bit
operations (`&`, `|`, `<<`, `>>`), just like flag manipulation in C.

**Worked example.** At the start, the white pawns are on a2 to h2. With our
square numbering (a1 = bit 0, ..., h1 = bit 7, a2 = bit 8, ..., h8 = bit 63)
those are bits 8 to 15:

```
rank 8  . . . . . . . .      bits 56..63
rank 7  . . . . . . . .
rank 6  . . . . . . . .
rank 5  . . . . . . . .
rank 4  . . . . . . . .
rank 3  . . . . . . . .      bits 16..23
rank 2  1 1 1 1 1 1 1 1      bits  8..15
rank 1  . . . . . . . .      bits  0..7
        a b c d e f g h

white_pawns = 0x0000_0000_0000_FF00
```

Shifting left by 8 moves every bit up one rank. AND-ing with the empty squares
keeps only the pushes that aren't blocked:

```rust
let single_pushes = (white_pawns << 8) & !occupied;   // 0x0000_0000_00FF_0000
```

That one line computes all eight pawn pushes at once, with no loop.

We check move generation with **perft**: count every possible position to a
fixed depth and compare against known correct numbers.

### 2. Judge a position (evaluation)

Give a position a score, e.g. "+1.5" means White is better by about one and a
half pawns.

- Simplest: count material (pawn 1, knight 3, bishop 3, rook 5, queen 9).
- Better: add positional ideas. **Piece-square tables** give each piece a bonus
  or penalty per square ("knights are better in the centre").
- Even better: king safety, pawn structure, mobility, and eventually a small
  neural network (**NNUE**) that learns evaluation from millions of games.

### 3. Look ahead (search)

Try my move, then your reply, then my next move, and so on. This builds a tree
of positions. Score the positions at the bottom with the evaluation, then work
back up assuming both sides pick their best move.

- **Minimax**: I pick the move that maximizes my score; my opponent picks the
  one that minimizes it.
- **Alpha-beta**: a trick that skips branches which clearly can't change the
  final answer. Same result as minimax, but searches much deeper in the same
  time.

**Worked example.** I have two moves, A and B. After each, my opponent has two
replies. The numbers are the evaluation (from my side) after each reply:

```
                 me (pick the max)
               /                \
          move A                 move B
     opp (pick the min)     opp (pick the min)
        /       \              /       \
       3         5            1         ?
```

- **Minimax:** after A, the opponent picks the reply worth 3 (the worse one for
  me), so A is worth 3. After B, the opponent can reach 1, so B is worth at most
  1, whatever "?" is. I play A.
- **Alpha-beta:** once A is known to be worth 3, I'm guaranteed at least 3
  (that's alpha). When B's first reply already gives 1, B can never beat 3, so
  we **skip "?" entirely**. Same answer, less work.

In a real game each position has around 35 moves and the tree is many plies
deep, so the skipped branches add up to millions of positions. Trying the best
moves first (move ordering) makes even more of these skips happen.

## Everything else is a refinement

Most of the advanced techniques make one of the three jobs faster or smarter:

- **Transposition table**: remember positions already analysed (the same
  position can be reached by different move orders).
- **Quiescence search**: at the end of the search, keep looking at captures so
  the engine doesn't stop calculating in the middle of a trade.
- **Move ordering**: try the most promising moves first, which makes alpha-beta
  skip far more branches.
- **Pruning and reductions**: search unpromising moves less deeply.

Each of these gets its own detailed concept note when we implement it. For
short definitions of every term in the plan, see the
[glossary](00-glossary.md).

## Further reading

- https://www.chessprogramming.org/Getting_Started
- https://www.chessprogramming.org/Elo_Rating
