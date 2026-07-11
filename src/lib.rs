//! rhll: a Rust HyperLogLog, exposed to Python via PyO3.
//!
//! [`HyperLogLogCore`] holds the algorithm, no PyO3 involved.
//! [`PyHyperLogLog`] is a thin `#[pyclass]` wrapper around it.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use xxhash_rust::xxh3::xxh3_64;

/// Valid range for the precision parameter `p`. `m = 2^p` registers.
const MIN_P: u8 = 4;
const MAX_P: u8 = 18;

/// Standard HLL bias-correction constant. Closed forms for small `m`,
/// general asymptotic formula for `m >= 128`. Ported from the Flajolet
/// et al. paper, cross-checked against `datasketch`'s implementation.
fn alpha(m: usize) -> f64 {
    match m {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / m as f64),
    }
}

/// Core HyperLogLog sketch. No Python/PyO3 dependency.
#[derive(Clone)]
pub struct HyperLogLogCore {
    p: u8,
    m: usize,
    registers: Vec<u8>,
}

impl HyperLogLogCore {
    pub fn new(p: u8) -> Result<Self, String> {
        if !(MIN_P..=MAX_P).contains(&p) {
            return Err(format!(
                "p must be between {MIN_P} and {MAX_P} inclusive, got {p}"
            ));
        }
        let m = 1usize << p;
        Ok(Self {
            p,
            m,
            registers: vec![0u8; m],
        })
    }

    pub fn p(&self) -> u8 {
        self.p
    }

    pub fn add(&mut self, data: &[u8]) {
        let hash = xxh3_64(data);

        // Top `p` bits select the register.
        let idx = (hash >> (64 - self.p as u32)) as usize;

        // Shift the index bits off the top so the remaining (64 - p) bits
        // land at the top of the word. leading_zeros() on that gives the
        // rank straight away; the zero-padded low bits only matter if the
        // whole remainder is zero, handled below.
        let remaining = hash << self.p;
        let max_rank = 64 - self.p + 1;
        let rank = if remaining == 0 {
            max_rank
        } else {
            (remaining.leading_zeros() as u8 + 1).min(max_rank)
        };

        if rank > self.registers[idx] {
            self.registers[idx] = rank;
        }
    }

    pub fn count(&self) -> f64 {
        let m = self.m as f64;
        let sum: f64 = self.registers.iter().map(|&r| 2f64.powi(-(r as i32))).sum();
        let raw = alpha(self.m) * m * m / sum;

        // Small-range correction: linear counting.
        if raw <= 2.5 * m {
            let zeros = self.registers.iter().filter(|&&r| r == 0).count();
            if zeros != 0 {
                return m * (m / zeros as f64).ln();
            }
            return raw;
        }

        // Large-range correction. Matters for 32-bit hashes near 2^32; with
        // a 64-bit hash this branch is basically dead at realistic scales.
        // Kept for completeness.
        let two_pow_64 = 2f64.powi(64);
        if raw > two_pow_64 / 30.0 {
            return -two_pow_64 * (1.0 - raw / two_pow_64).ln();
        }

        raw
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        if self.p != other.p {
            return Err(format!(
                "cannot merge sketches with different p ({} vs {})",
                self.p, other.p
            ));
        }
        for i in 0..self.m {
            if other.registers[i] > self.registers[i] {
                self.registers[i] = other.registers[i];
            }
        }
        Ok(())
    }

    pub fn registers(&self) -> &[u8] {
        &self.registers
    }

    /// Serialize as `[p as u8][registers...]`. Simple, not compressed.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.registers.len());
        out.push(self.p);
        out.extend_from_slice(&self.registers);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let &p = bytes.first().ok_or("empty byte buffer")?;
        let mut hll = Self::new(p)?;
        let registers = &bytes[1..];
        if registers.len() != hll.m {
            return Err(format!(
                "expected {} register bytes for p={}, got {}",
                hll.m,
                p,
                registers.len()
            ));
        }
        hll.registers.copy_from_slice(registers);
        Ok(hll)
    }
}

/// Thin PyO3 wrapper around [`HyperLogLogCore`].
#[pyclass(name = "HyperLogLog")]
pub struct PyHyperLogLog {
    inner: HyperLogLogCore,
}

#[pymethods]
impl PyHyperLogLog {
    #[new]
    #[pyo3(signature = (p=12))]
    fn new(p: u8) -> PyResult<Self> {
        HyperLogLogCore::new(p)
            .map(|inner| Self { inner })
            .map_err(PyValueError::new_err)
    }

    /// Add a single item.
    fn update(&mut self, data: &[u8]) {
        self.inner.add(data);
    }

    /// Convenience: `str` -> utf8 -> `update`, avoids `.encode()` at call sites.
    fn update_str(&mut self, s: &str) {
        self.inner.add(s.as_bytes());
    }

    fn count(&self) -> f64 {
        self.inner.count()
    }

    /// In-place union with another sketch of the same `p`.
    fn merge(&mut self, other: &PyHyperLogLog) -> PyResult<()> {
        self.inner
            .merge(&other.inner)
            .map_err(PyValueError::new_err)
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    /// Raw register bytes, exposed for debugging/serialization.
    fn registers<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.inner.registers())
    }

    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.to_bytes())
    }

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        HyperLogLogCore::from_bytes(data)
            .map(|inner| Self { inner })
            .map_err(PyValueError::new_err)
    }

    fn __len__(&self) -> usize {
        self.inner.m
    }

    fn __repr__(&self) -> String {
        format!("HyperLogLog(p={})", self.inner.p())
    }
}

#[pymodule]
fn _rhll(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHyperLogLog>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sketch_counts_zero() {
        let hll = HyperLogLogCore::new(12).unwrap();
        assert_eq!(hll.count(), 0.0);
    }

    #[test]
    fn rejects_invalid_p() {
        assert!(HyperLogLogCore::new(MIN_P - 1).is_err());
        assert!(HyperLogLogCore::new(MAX_P + 1).is_err());
        assert!(HyperLogLogCore::new(MIN_P).is_ok());
        assert!(HyperLogLogCore::new(MAX_P).is_ok());
    }

    fn estimate_error(n: usize, p: u8) -> f64 {
        let mut hll = HyperLogLogCore::new(p).unwrap();
        for i in 0..n {
            hll.add(format!("item-{i}").as_bytes());
        }
        (hll.count() - n as f64).abs() / n as f64
    }

    #[test]
    fn estimate_within_error_bounds_1k() {
        assert!(estimate_error(1_000, 14) < 0.05);
    }

    #[test]
    fn estimate_within_error_bounds_100k() {
        assert!(estimate_error(100_000, 14) < 0.05);
    }

    #[test]
    fn duplicates_do_not_inflate_count() {
        let mut hll = HyperLogLogCore::new(14).unwrap();
        for _ in 0..5 {
            for i in 0..10_000 {
                hll.add(format!("item-{i}").as_bytes());
            }
        }
        let error = (hll.count() - 10_000.0).abs() / 10_000.0;
        assert!(error < 0.05, "error={error}");
    }

    #[test]
    fn merge_disjoint_sketches_approximates_union() {
        let mut a = HyperLogLogCore::new(14).unwrap();
        let mut b = HyperLogLogCore::new(14).unwrap();
        for i in 0..50_000 {
            a.add(format!("a-{i}").as_bytes());
        }
        for i in 0..50_000 {
            b.add(format!("b-{i}").as_bytes());
        }
        a.merge(&b).unwrap();
        let error = (a.count() - 100_000.0).abs() / 100_000.0;
        assert!(error < 0.05, "error={error}");
    }

    #[test]
    fn merge_requires_matching_p() {
        let mut a = HyperLogLogCore::new(10).unwrap();
        let b = HyperLogLogCore::new(12).unwrap();
        assert!(a.merge(&b).is_err());
    }

    #[test]
    fn to_bytes_from_bytes_round_trip() {
        let mut hll = HyperLogLogCore::new(10).unwrap();
        for i in 0..1_000 {
            hll.add(format!("item-{i}").as_bytes());
        }
        let restored = HyperLogLogCore::from_bytes(&hll.to_bytes()).unwrap();
        assert_eq!(hll.registers(), restored.registers());
        assert_eq!(hll.count(), restored.count());
    }
}
