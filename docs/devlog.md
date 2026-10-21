# Devlog

One entry per step: what changed, which files, why, and the result. Newest at
the bottom.

## Step 0: Design and planning (2026-10-21)

**What:** Agreed on the overall design and roadmap. No code yet.

**Decisions:**
- Engine in Rust, learning as we go.
- UCI engine first, web UI (WASM + TypeScript) later.
- Aim for maximum strength, built phase by phase, with each optimization
  measured.
- Document every step in this devlog and in `docs/concepts/`.

**Files:**
- `docs/design.md`: design plan and roadmap.
- `docs/devlog.md`: this file.
- `docs/concepts/00-glossary.md`: plain-language definitions of every term in
  the plan.
- `docs/concepts/01-how-chess-engines-work.md`: overview of Elo and the three
  jobs of an engine, with worked examples.
- `README.md`: project description and links to the docs.

**Concepts:** [Glossary](concepts/00-glossary.md),
[How chess engines work](concepts/01-how-chess-engines-work.md)

**Next:** Phase 0, setting up the Rust workspace.
