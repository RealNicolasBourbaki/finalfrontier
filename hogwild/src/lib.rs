extern crate ndarray;

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use ndarray::{Array, ArrayView, ArrayViewMut, Axis, Dimension, Ix, Ix1, Ix2, Ix3, RemoveAxis};

/// Array for Hogwild parallel optimization.
///
/// This array type can be used for the Hogwild (Niu, et al. 2011) method
/// of parallel Stochastic Gradient descent. In Hogwild different threads
/// share the same parameters without locking. If SGD is performed on a
/// sparse optimization problem, where only a small subset of parameters
/// is updated in each gradient descent, the impact of data races is
/// negligible.
///
/// In order to use Hogwild in Rust, we have to subvert the ownership
/// system. This is what the `HogwildArray` type does. It uses reference
/// counting to share an *ndarray* `Array` type between multiple
/// `HogwildArray` instances. Views of the underling `Array` can be borrowed
/// mutably from each instance, without mutual exclusion between mutable
/// borrows in different `HogwildArray` instances.
///
/// # Example
///
/// ```
/// extern crate hogwild;
/// extern crate ndarray;
///
/// use hogwild::HogwildArray2;
/// use ndarray::Array2;
///
/// let mut a1: HogwildArray2<f32> = Array2::zeros((2, 2)).into();
/// let mut a2 = a1.clone();
///
/// let mut a1_view = a1.view_mut();
///
/// let c00 = &mut a1_view[(0, 0)];
/// *c00 = 1.0;
///
/// // Two simultaneous mutable borrows of the underlying array.
/// a2.view_mut()[(1, 1)] = *c00 * 2.0;
///
/// assert_eq!(&[1.0, 0.0, 0.0, 2.0], a2.as_slice().unwrap());
/// ```

#[derive(Clone)]
pub struct HogwildArray<A, D>(Arc<UnsafeCell<Array<A, D>>>);

impl<A, D> HogwildArray<A, D> {
    #[inline]
    fn as_mut(&mut self) -> &mut Array<A, D> {
        let ptr = self.0.as_ref().get();
        unsafe { &mut *ptr }
    }

    #[inline]
    fn as_ref(&self) -> &Array<A, D> {
        let ptr = self.0.as_ref().get();
        unsafe { &*ptr }
    }
}

impl<A, D> HogwildArray<A, D>
where
    D: Dimension + RemoveAxis,
{
    /// Get an immutable subview of the Hogwild array.
    #[inline]
    pub fn subview(&self, axis: Axis, index: Ix) -> ArrayView<A, D::Smaller> {
        self.as_ref().subview(axis, index)
    }

    /// Get a mutable subview of the Hogwild array.
    #[inline]
    pub fn subview_mut(&mut self, axis: Axis, index: Ix) -> ArrayViewMut<A, D::Smaller> {
        self.as_mut().subview_mut(axis, index)
    }
}

impl<A, D> HogwildArray<A, D>
where
    D: Dimension,
{
    /// Get a slice reference to the underlying data array.
    #[inline]
    pub fn as_slice(&self) -> Option<&[A]> {
        self.as_ref().as_slice()
    }

    /// Get an immutable view of the Hogwild array.
    #[inline]
    pub fn view(&self) -> ArrayView<A, D> {
        self.as_ref().view()
    }

    /// Get an mutable view of the Hogwild array.
    #[inline]
    pub fn view_mut(&mut self) -> ArrayViewMut<A, D> {
        self.as_mut().view_mut()
    }
}

impl<A, D> From<Array<A, D>> for HogwildArray<A, D> {
    fn from(a: Array<A, D>) -> Self {
        HogwildArray(Arc::new(UnsafeCell::new(a)))
    }
}

unsafe impl<A, D> Send for HogwildArray<A, D> {}

unsafe impl<A, D> Sync for HogwildArray<A, D> {}

/// One-dimensional Hogwild array.
pub type HogwildArray1<A> = HogwildArray<A, Ix1>;

/// Two-dimensional Hogwild array.
pub type HogwildArray2<A> = HogwildArray<A, Ix2>;

/// Three-dimensional Hogwild array.
pub type HogwildArray3<A> = HogwildArray<A, Ix3>;

/// Hogwild for arbitrary data types.
///
/// `Hogwild` subverts Rust's type system by allowing concurrent modification
/// of values. This should only be used for data types that cannot end up in
/// an inconsistent state due to data races. For arrays `HogwildArray` should
/// be preferred.
#[derive(Clone)]
pub struct Hogwild<T>(Arc<UnsafeCell<T>>);

impl<T> Default for Hogwild<T>
where
    T: Default,
{
    fn default() -> Self {
        Hogwild(Arc::new(UnsafeCell::new(T::default())))
    }
}

impl<T> Deref for Hogwild<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let ptr = self.0.as_ref().get();
        unsafe { &*ptr }
    }
}

impl<T> DerefMut for Hogwild<T> {
    fn deref_mut(&mut self) -> &mut T {
        let ptr = self.0.as_ref().get();
        unsafe { &mut *ptr }
    }
}

unsafe impl<T> Send for Hogwild<T> {}

unsafe impl<T> Sync for Hogwild<T> {}

#[cfg(test)]
mod test {
    use ndarray::Array2;

    use super::{Hogwild, HogwildArray2};

    #[test]
    pub fn hogwild_test() {
        let mut a1: Hogwild<usize> = Hogwild::default();
        let mut a2 = a1.clone();

        *a1 = 1;
        assert_eq!(*a2, 1);
        *a2 = 2;
        assert_eq!(*a1, 2);
    }

    #[test]
    pub fn hogwild_array_test() {
        let mut a1: HogwildArray2<f32> = Array2::zeros((2, 2)).into();
        let mut a2 = a1.clone();

        let mut a1_view = a1.view_mut();

        let c00 = &mut a1_view[(0, 0)];
        *c00 = 1.0;

        // Two simultaneous mutable borrows of the underlying array.
        a2.view_mut()[(1, 1)] = *c00 * 2.0;

        assert_eq!(&[1.0, 0.0, 0.0, 2.0], a2.as_slice().unwrap());
    }
}