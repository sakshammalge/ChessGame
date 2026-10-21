# Continuous Integration (CI)

Used in: Phase 0 (setup). Runs on every pull request from now on.

## 1. The problem

Over the next phases we'll change hundreds of lines at a time: new move
generation, new search tricks. Every change can break something that worked
before. In chess engines the breakage is often silent: a move generator that
misses one en passant case still plays chess, it just plays slightly wrong
chess, and you might not notice for weeks.

If we rely on remembering to run the tests before each merge, sooner or later
we'll forget. And "it passed on my laptop" doesn't prove it passes on a clean
machine.

## 2. The idea in plain words

**Continuous integration** means a server automatically builds and tests the
code every time someone pushes or opens a pull request. If anything fails,
the PR gets a red X and we know before merging.

GitHub has this built in, called **GitHub Actions**. You describe the checks
in a YAML file under `.github/workflows/`, and GitHub runs them on a fresh
Linux virtual machine each time.

Some vocabulary:

- **Workflow**: one YAML file, describing when to run and what to run.
- **Trigger** (`on:`): which events start the workflow, e.g. a push to `main`
  or any pull request.
- **Job**: a group of steps that run on one machine (a **runner**).
- **Step**: one command (`run:`) or one reusable **action** (`uses:`), which is
  a packaged step someone else wrote, like a library function.

## 3. Worked example: what happens when you open a PR

```
 you push a branch and open a PR
              |
              v
 GitHub reads .github/workflows/ci.yml, sees "on: pull_request"
              |
              v
 starts a fresh ubuntu-latest machine
              |
              v
 1. checkout        copy the repo onto the machine
 2. install Rust    stable + rustfmt + clippy
 3. restore cache   reuse compiled dependencies from last time
 4. cargo fmt --check           is the code formatted?
 5. cargo clippy                any lint warnings?
 6. cargo test                  do the tests pass (debug)?
 7. cargo test --release        do they pass with optimizations on?
              |
     any step fails? ──yes──> red X on the PR, the log shows which step
              |
              no
              v
        green check
```

Steps run in order and stop at the first failure, so the cheap checks go
first.

## 4. How it looks in our code

`.github/workflows/ci.yml`, piece by piece:

```yaml
on:
  push:
    branches: [main]
  pull_request:
```

Run on every push to `main` (so `main` itself is always checked) and on every
pull request.

```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

If you push twice quickly to the same branch, cancel the older run. Only the
newest code matters. `${{ ... }}` inserts a value GitHub provides, here the
branch name.

```yaml
env:
  RUSTFLAGS: -D warnings
```

`RUSTFLAGS` passes extra flags to every `rustc` call. `-D warnings` means
"deny warnings": treat every warning as an error, like `-Werror` in gcc. Locally
you can see warnings and keep going; in CI they fail the build, so they get
fixed instead of piling up.

```yaml
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
```

Three ready-made actions: copy the code, install Rust, and cache the build
output. `@v4` pins the version of the action, so a new release can't
surprise us. The cache matters because compiling dependencies from scratch on
every run gets slow once we add some.

```yaml
      - name: Check formatting
        run: cargo fmt --all --check
```

`--check` doesn't change any files. It fails if any file isn't formatted the
way `cargo fmt` would format it. Fix locally with `cargo fmt`.

```yaml
      - name: Clippy (lints)
        run: cargo clippy --workspace --all-targets
```

`--all-targets` includes tests and the tool programs, not just the libraries.
Clippy catches things like needless clones, `x == true`, and loops that could
be simpler. Each message links to an explanation, which is a good way to
learn idiomatic Rust.

```yaml
      - name: Tests (debug build)
        run: cargo test --workspace
      - name: Tests (release build)
        run: cargo test --workspace --release
```

Why test twice? Debug and release builds really behave differently (see
[the Rust toolchain note](02-rust-toolchain-and-cargo.md)): debug checks for
integer overflow, release doesn't, and the optimizer can expose bugs that
only show up when code is reordered. Later, perft will run here too, and deep
perft is only practical in release mode.

## 5. Pitfalls

- **Run the same checks locally before pushing.** It's faster than waiting for
  CI. The full set is:

  ```text
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```

- **A new Rust release adds new Clippy lints.** We use `stable`, so a passing
  PR can occasionally turn red a few weeks later without any change of ours.
  The fix is usually a small, mechanical code change that Clippy suggests.
- **CI only tests what we wrote tests for.** A green check means "nothing we
  test for broke", not "no bugs". That's why perft (phase 1) and the test
  suites (phase 3) matter so much.
- **Don't make CI too slow.** If a check takes 20 minutes, people stop waiting
  for it. Long engine matches (SPRT) will run separately, not on every push.

## 6. Measured effect

On this PR the four checks pass: formatting, Clippy with warnings as errors,
4 tests in debug and 4 in release. Locally, the whole sequence takes a few
seconds once dependencies are compiled. The actual CI run time is shown on the
PR's "Checks" tab.

## 7. Further reading

- GitHub Actions quickstart:
  https://docs.github.com/en/actions/writing-workflows/quickstart
- Workflow syntax reference:
  https://docs.github.com/en/actions/writing-workflows/workflow-syntax-for-github-actions
- `dtolnay/rust-toolchain`: https://github.com/dtolnay/rust-toolchain
- `Swatinem/rust-cache`: https://github.com/Swatinem/rust-cache
- Continuous integration in general:
  https://en.wikipedia.org/wiki/Continuous_integration
