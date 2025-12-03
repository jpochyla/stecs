#![feature(portable_simd)]

use std::simd::{Simd, num::SimdUint};

fn main() {
    divan::main();
}

fn count_ones_simple(x: Simd<u64, 4>) -> usize {
    let mut sum = 0;
    for i in x.to_array() {
        sum += u64::count_ones(i);
    }
    sum as usize
}

fn count_ones_simd(x: Simd<u64, 4>) -> usize {
    SimdUint::count_ones(x).reduce_sum() as usize
}

#[divan::bench]
fn simple_all_zeros() -> usize {
    let x = Simd::from_array([0u64; 4]);
    count_ones_simple(x)
}

#[divan::bench]
fn simd_all_zeros() -> usize {
    let x = Simd::from_array([0u64; 4]);
    count_ones_simd(x)
}

#[divan::bench]
fn simple_all_ones() -> usize {
    let x = Simd::from_array([u64::MAX; 4]);
    count_ones_simple(x)
}

#[divan::bench]
fn simd_all_ones() -> usize {
    let x = Simd::from_array([u64::MAX; 4]);
    count_ones_simd(x)
}

#[divan::bench]
fn simple_mixed() -> usize {
    let x = Simd::from_array([
        0x123456789ABCDEF0,
        0xFEDCBA9876543210,
        0xAAAAAAAAAAAAAAAA,
        0x5555555555555555,
    ]);
    count_ones_simple(x)
}

#[divan::bench]
fn simd_mixed() -> usize {
    let x = Simd::from_array([
        0x123456789ABCDEF0,
        0xFEDCBA9876543210,
        0xAAAAAAAAAAAAAAAA,
        0x5555555555555555,
    ]);
    count_ones_simd(x)
}

#[divan::bench]
fn simple_sparse() -> usize {
    let x = Simd::from_array([1u64, 2, 4, 8]);
    count_ones_simple(x)
}

#[divan::bench]
fn simd_sparse() -> usize {
    let x = Simd::from_array([1u64, 2, 4, 8]);
    count_ones_simd(x)
}
