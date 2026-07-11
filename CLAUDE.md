# CLAUDE.md

Guidance for working in this repo.

## Overview

`rhll` is a Rust HyperLogLog exposed to Python via PyO3/maturin. The core
algorithm lives in `src/lib.rs` as a plain Rust struct (`HyperLogLogCore`,
no PyO3 dependency); `PyHyperLogLog` is a thin `#[pyclass]` wrapper around
it. See `rhll-project-plan.md` for the full design rationale, algorithm
notes, and phased build plan. Read it before making non-trivial changes to
the algorithm or public API.

## Project layout

```
rhll/
├── Cargo.toml
├── pyproject.toml            # maturin config
├── src/
│   └── lib.rs                # HyperLogLogCore (plain Rust) + PyO3 bindings
├── python/
│   └── rhll/
│       ├── __init__.py
│       └── __init__.pyi      # type stubs for the compiled extension
├── tests/                    # pytest suite
├── benchmarks/                # rhll vs datasketch vs HLL
├── .github/workflows/
│   ├── ci.yml                 # rust fmt/clippy/test, python test matrix, ruff
│   └── release.yml            # build wheels (linux/macos/windows + sdist), publish
└── README.md
```

## Development commands

```sh
# Rust unit tests (pure algorithm, no PyO3 linking issues):
cargo test --no-default-features

# Build + install into the current Python env, then run the Python suite:
python -m venv .venv && source .venv/bin/activate
pip install -e '.[dev]'
maturin develop --release
pytest

# Lint (same checks CI runs):
cargo fmt --check
cargo clippy --no-default-features -- -D warnings
ruff check .
ruff format --check .

# Benchmarks (rhll vs datasketch vs HLL):
python benchmarks/bench_insert.py
python benchmarks/bench_count.py
```

Note: `extension-module` (the pyo3 feature that links against libpython) is
gated behind a Cargo feature that's on by default, so plain `cargo build`/
`cargo test` will try to link libpython and fail outside a Python venv
context unless you pass `--no-default-features`.

## Conventions

- Core algorithm changes go in `HyperLogLogCore` with Rust unit tests in the
  `#[cfg(test)]` module at the bottom of `src/lib.rs`; keep it PyO3-free.
- `PyHyperLogLog` should stay a thin wrapper: no algorithm logic there,
  just type/error marshalling (`PyResult`, `PyValueError`).
- Any behavior change to the public Python API needs a matching update to
  `python/rhll/__init__.pyi`.
- After changing `src/lib.rs`, rebuild with `maturin develop --release`
  before running `pytest`. An editable install does not pick up Rust
  changes automatically.
- `pyo3` is built with `abi3-py311` (`Cargo.toml`), matching
  `requires-python` in `pyproject.toml`. One wheel per platform covers
  every supported Python version; don't drop `abi3-pyXXX` without also
  widening the release matrix in `release.yml` to build per-version
  wheels again.
- Keep the `version` field in `Cargo.toml` and `pyproject.toml` in sync.
  They aren't derived from each other.

## Releasing

`release.yml` builds wheels for Linux (x86_64/aarch64, manylinux),
Windows (x64), macOS (x86_64/aarch64), plus an sdist.

- `workflow_dispatch` (manual run from the Actions tab) publishes the
  build to TestPyPI as a dry run.
- Pushing a tag matching `v*` (e.g. `v0.1.0`) publishes to real PyPI.

Both publish jobs use PyPI Trusted Publishing (OIDC) via
`pypa/gh-action-pypi-publish`, not a stored API token. Before either can
succeed, a trusted publisher must be configured on the PyPI/TestPyPI side
for this repo: project settings → Publishing → add GitHub Actions
publisher, pointing at owner `f4hy`, repo `rhll`, workflow `release.yml`,
and environment `pypi` (or `testpypi`). That's a one-time manual step on
pypi.org / test.pypi.org, not something this repo's config can do for you.
`pypi` is configured (v0.1.0 published successfully); `testpypi` is not
yet set up, so `workflow_dispatch` dry runs currently fail at the token
exchange with `invalid-publisher` — harmless (fails before any upload)
but means the dry-run path isn't actually verifying anything until
someone adds that trusted publisher on test.pypi.org too.

Before tagging a real release: bump `version` in both `Cargo.toml` and
`pyproject.toml` (they moved together for v0.1.0: both were `0.1.0`), and
ideally confirm a TestPyPI dry run installs and imports cleanly first
(see the gap noted above).
