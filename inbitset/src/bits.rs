use std::{
    mem,
    simd::{Simd, num::SimdUint},
};

type Scalar = u64;

const SIMD_LANES: usize = 4;
const SCALAR_BITS: usize = mem::size_of::<Scalar>() * 8;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub(crate) struct Bits {
    bits: Simd<Scalar, SIMD_LANES>,
}

impl Bits {
    /// Returns true if the block does not have any set bits.
    pub(crate) fn is_empty(&self) -> bool {
        self.bits.reduce_or() == 0
    }

    /// Returns the number of set bits in the block.
    pub(crate) fn count_set(&self) -> usize {
        self.bits.count_ones().reduce_sum() as usize
    }

    /// Execute `f` for each set bit, from the lowest to highest, passing it the bit index.
    pub(crate) fn for_each_set(&self, mut f: impl FnMut(usize)) {
        for lane in 0..SIMD_LANES {
            let mut value = self.bits[lane];
            while value != 0 {
                let bit = value.trailing_zeros() as usize;
                let bit_index = lane * SCALAR_BITS + bit;
                f(bit_index);
                value &= value - 1; // Clear the lowest set bit.
            }
        }
    }

    /// Returns the value of the bit at the given index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `bit_index` is less than `Self::SIZE`.
    pub(crate) unsafe fn get_unchecked(&self, bit_index: usize) -> bool {
        let lane = bit_index / SCALAR_BITS;
        let bit = bit_index % SCALAR_BITS;
        let mask = 1 << bit;
        (self.bits[lane] & mask) != 0
    }

    /// Sets the bit at the given index without bounds checking. Returns the previous value.
    ///
    /// # Safety
    /// The caller must ensure that `bit_index` is less than `Self::SIZE`.
    pub(crate) unsafe fn set_unchecked(&mut self, bit_index: usize) -> bool {
        let lane = bit_index / SCALAR_BITS;
        let bit = bit_index % SCALAR_BITS;
        let mask = 1 << bit;
        let old = (self.bits[lane] & mask) != 0;
        self.bits[lane] |= mask;
        old
    }

    /// Unsets the bit at the given index without bounds checking. Returns the previous value.
    ///
    /// # Safety
    /// The caller must ensure that `bit_index` is less than `Self::SIZE`.
    pub(crate) unsafe fn unset_unchecked(&mut self, bit_index: usize) -> bool {
        let lane = bit_index / SCALAR_BITS;
        let bit = bit_index % SCALAR_BITS;
        let mask = 1 << bit;
        let old = (self.bits[lane] & mask) != 0;
        self.bits[lane] &= !mask;
        old
    }

    pub(crate) fn take_empty_block(&mut self) -> usize {
        let array = self.bits.as_mut_array();
        let empty = array[0];
        array[0] = 0;
        empty as usize
    }

    pub(crate) fn set_empty_block(&mut self, empty: usize) {
        let array = self.bits.as_mut_array();
        array[0] = empty as Scalar;
    }
}
