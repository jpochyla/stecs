#![feature(portable_simd)]

mod bits;

use std::mem::MaybeUninit;

use bits::*;

struct BitSet {
    top: TopBlock,
    middle: Level<MiddleBlock>,
    bottom: Level<BottomBlock>,
}

impl BitSet {
    pub fn insert(&mut self, bit_index: usize) {
        let indices = LevelIndices::new(bit_index);

        unsafe {
            self.get_or_insert_bottom_block(indices.top, indices.middle)
                .bits
                .set_unchecked(indices.bottom);
        }
    }

    pub fn contains(&self, bit_index: usize) {
        let indices = LevelIndices::new(bit_index);

        unsafe {
            self.bottom
                .blocks
                .get_unchecked(indices.middle)
                .bits
                .get_unchecked(indices.bottom)
        }
    }

    unsafe fn get_or_insert_bottom_block(
        &mut self,
        top_index: usize,
        middle_index: usize,
    ) -> &mut BottomBlock {
        unsafe {
            let middle_block_index = self
                .top
                .get_or_insert(top_index, || self.middle.insert_block());

            let bottom_block_index = self
                .middle
                .blocks
                .get_unchecked_mut(middle_block_index)
                .get_or_insert(middle_index, || self.bottom.insert_block());

            self.bottom.blocks.get_unchecked_mut(bottom_block_index)
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
struct Level<B> {
    blocks: Vec<B>,
    empty: usize, // usize::MAX when N/A.
}

impl<B> Level<B>
where
    B: Block,
{
    fn pop_empty_block(&mut self) -> Option<usize> {
        if self.empty == usize::MAX {
            return None;
        }
        let index = self.empty;
        unsafe {
            self.empty = self
                .blocks
                .get_unchecked_mut(index)
                .bits_mut()
                .take_empty_block();
        }
        Some(index)
    }

    fn insert_block(&mut self) -> usize {
        if let Some(index) = self.pop_empty_block() {
            index
        } else {
            let index = self.blocks.len();
            self.blocks.push(B::default());
            index
        }
    }
}

trait Block: Default {
    fn bits_mut(&mut self) -> &mut Bits;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct TopBlock {
    bits: Bits,
    indices: [u8; 256],
}

impl TopBlock {
    unsafe fn get_or_insert(&mut self, bit_index: usize, f: impl FnOnce() -> usize) -> usize {
        unsafe {
            let exists = self.bits.set_unchecked(bit_index);
            if exists {
                let compressed = *self.indices.get_unchecked(bit_index);
                let block_index = compressed
                    .try_into()
                    .expect("failed to decompress block index");
                block_index
            } else {
                let block_index = f();
                let compressed = block_index
                    .try_into()
                    .expect("failed to compress block index");
                *self.indices.get_unchecked_mut(bit_index) = compressed;
                block_index
            }
        }
    }
}

impl Block for TopBlock {
    fn bits_mut(&mut self) -> &mut Bits {
        &mut self.bits
    }
}

impl Default for TopBlock {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct MiddleBlock {
    bits: Bits,
    indices: [u16; 256],
}

impl MiddleBlock {
    unsafe fn get_or_insert(&mut self, bit_index: usize, f: impl FnOnce() -> usize) -> usize {
        unsafe {
            let exists = self.bits.set_unchecked(bit_index);
            if exists {
                let compressed = *self.indices.get_unchecked(bit_index);
                let block_index = compressed
                    .try_into()
                    .expect("failed to decompress block index");
                block_index
            } else {
                let block_index = f();
                let compressed = block_index
                    .try_into()
                    .expect("failed to compress block index");
                *self.indices.get_unchecked_mut(bit_index) = compressed;
                block_index
            }
        }
    }
}

impl Block for MiddleBlock {
    fn bits_mut(&mut self) -> &mut Bits {
        &mut self.bits
    }
}

impl Default for MiddleBlock {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
struct BottomBlock {
    bits: Bits,
}

impl Block for BottomBlock {
    fn bits_mut(&mut self) -> &mut Bits {
        &mut self.bits
    }
}

#[derive(Copy, Clone, Debug)]
struct LevelIndices {
    top: usize,
    middle: usize,
    bottom: usize,
}

impl LevelIndices {
    const fn new(bit_index: usize) -> Self {
        const BOTTOM_BLOCK_CAP: usize = 1 << 8;
        const MIDDLE_BLOCK_CAP: usize = 1 << 16;

        let top = bit_index / MIDDLE_BLOCK_CAP;
        let top_rem = bit_index % MIDDLE_BLOCK_CAP;
        let middle = top_rem / BOTTOM_BLOCK_CAP;
        let middle_rem = top_rem % BOTTOM_BLOCK_CAP;
        let bottom = middle_rem;

        Self {
            top,
            middle,
            bottom,
        }
    }
}
