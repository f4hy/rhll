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
| 10,000 | 0.001s | 0.014s | 0.001s |
| 100,000 | 0.005s | 0.123s | 0.009s |
| 1,000,000 | 0.057s | 1.288s | 0.086s |
| 10,000,000 | 0.555s | 12.399s | 0.891s |

`rhll` is ~22x faster than `datasketch` at 10M inserts, and modestly faster
than the `HLL` C extension. Likely the `xxh3_64` hash vs. `datasketch`'s
SHA1, plus avoiding the Python-level overhead per `.update()` call that
`datasketch` pays (NumPy array indexing, etc). Per-call PyO3/CPython
overhead dominates at these item counts, which is also why the debug and
release builds perform almost identically: the hot loop is call overhead,
not the Rust-side arithmetic. The gap over `datasketch` widens slightly at
larger n (~19x at 1M, ~22x at 10M) as SHA1's per-call cost adds up.

## Count accuracy + speed

`python benchmarks/bench_count.py`, 10,000,000 distinct items, each
inserted 3x (30,000,000 total updates), `p=14`:

| library | estimate | true | error % | insert time | count time |
|---|---|---|---|---|---|
| rhll | 10,064,440 | 10,000,000 | 0.64% | 2.435s | 0.00021s |
| datasketch | 10,056,563 | 10,000,000 | 0.57% | 39.471s | 0.00027s |
| HLL | 10,077,127 | 10,000,000 | 0.77% | 3.389s | 0.00001s |

All three land within the expected ~1-2% error bound for `p=14`
(theoretical relative error ≈ `1.04/sqrt(m)` ≈ 0.4% for m=16384, single-run
variance pushes the observed error a bit above that) and are close to each
other, as expected: same algorithm, different hash functions and
correction-constant implementations.

## Reproducing

```sh
python -m venv .venv && source .venv/bin/activate
pip install -e '.[dev]'
maturin develop --release   # release build for realistic benchmark numbers
python benchmarks/bench_insert.py
python benchmarks/bench_count.py
```
