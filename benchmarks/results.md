# Benchmark results

Machine / environment:

- CPU: Intel(R) Xeon(R) W-2123 @ 3.60GHz (8 threads), Linux 7.1.3-arch1-2 x86_64
- Python: 3.14.6
- Rust: rustc 1.97.0
- `rhll`: 0.1.0 (this commit)
- `datasketch`: 2.0.0
- `HLL`: 3.0.0

Release build (`maturin develop --release`), single run, single-threaded,
no warmup. Treat as directional, not statistically rigorous (no repeated
trials / variance reported yet).

## Insert throughput

`python benchmarks/bench_insert.py`

| n | rhll | datasketch | HLL |
|---|---|---|---|
| 10,000 | 0.001s | 0.012s | 0.001s |
| 100,000 | 0.006s | 0.121s | 0.010s |
| 1,000,000 | 0.069s | 1.300s | 0.082s |

`rhll` is ~19x faster than `datasketch` at 1M inserts, and modestly faster
than the `HLL` C extension. Likely the `xxh3_64` hash vs. `datasketch`'s
SHA1, plus avoiding the Python-level overhead per `.update()` call that
`datasketch` pays (NumPy array indexing, etc). Per-call PyO3/CPython
overhead dominates at this item count, which is also why the debug and
release builds perform almost identically here: the hot loop is call
overhead, not the Rust-side arithmetic.

## Count accuracy + speed

`python benchmarks/bench_count.py`, 100,000 distinct items, each inserted
3x (300,000 total updates), `p=14`:

| library | estimate | true | error % | insert time | count time |
|---|---|---|---|---|---|
| rhll | 99,797 | 100,000 | 0.20% | 0.027s | 0.00020s |
| datasketch | 99,682 | 100,000 | 0.32% | 0.406s | 0.00022s |
| HLL | 100,108 | 100,000 | 0.11% | 0.033s | 0.00001s |

All three land well within the expected ~1-2% error bound for `p=14`
(theoretical relative error ≈ `1.04/sqrt(m)` ≈ 0.4% for m=16384) and are
close to each other, as expected: same algorithm, different hash functions
and correction-constant implementations.

## Reproducing

```sh
python -m venv .venv && source .venv/bin/activate
pip install -e '.[dev]'
maturin develop --release   # release build for realistic benchmark numbers
python benchmarks/bench_insert.py
python benchmarks/bench_count.py
```
