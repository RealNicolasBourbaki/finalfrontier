pub static EOS: &str = "</s>";

/// Tolerance for small negative values.
const NEGATIVE_TOLERANCE: f32 = 1e-5;

/// Add a small value, to prevent returning Inf on underflow.
#[inline]
pub fn safe_ln(v: f32) -> f32 {
    (v + NEGATIVE_TOLERANCE).ln()
}

#[cfg(test)]
pub use self::test::*;

#[cfg(test)]
mod test {
    use ndarray::{ArrayView, Dimension};

    pub fn close(a: f32, b: f32, eps: f32) -> bool {
        let diff = (a - b).abs();
        if diff > eps {
            return false;
        }

        true
    }

    pub fn all_close(a: &[f32], b: &[f32], eps: f32) -> bool {
        for (&av, &bv) in a.iter().zip(b) {
            if !close(av, bv, eps) {
                return false;
            }
        }

        true
    }

    pub fn array_all_close<Ix>(a: ArrayView<f32, Ix>, b: ArrayView<f32, Ix>, eps: f32) -> bool
    where
        Ix: Dimension,
    {
        for (&av, &bv) in a.iter().zip(b) {
            if !close(av, bv, eps) {
                return false;
            }
        }

        true
    }
}