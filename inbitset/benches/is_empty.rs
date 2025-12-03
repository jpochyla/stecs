#![feature(portable_simd)]

use std::simd::{Simd, num::SimdUint};

fn main() {
    divan::main();
}

fn is_empty_reduce(x: Simd<u64, 4>) -> bool {
    x.reduce_or() == 0
}

fn is_empty_or(x: Simd<u64, 4>) -> bool {
    x[0] | x[1] | x[2] | x[3] == 0
}

#[divan::bench]
fn cmp_all_zeros() -> bool {
    let x = Simd::from_array([0u64; 4]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_all_zeros() -> bool {
    let x = Simd::from_array([0u64; 4]);
    is_empty_or(x)
}

#[divan::bench]
fn cmp_all_ones() -> bool {
    let x = Simd::from_array([u64::MAX; 4]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_all_ones() -> bool {
    let x = Simd::from_array([u64::MAX; 4]);
    is_empty_or(x)
}

#[divan::bench]
fn cmp_mixed() -> bool {
    let x = Simd::from_array([
        0x123456789ABCDEF0,
        0xFEDCBA9876543210,
        0xAAAAAAAAAAAAAAAA,
        0x5555555555555555,
    ]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_mixed() -> bool {
    let x = Simd::from_array([
        0x123456789ABCDEF0,
        0xFEDCBA9876543210,
        0xAAAAAAAAAAAAAAAA,
        0x5555555555555555,
    ]);
    is_empty_or(x)
}

#[divan::bench]
fn cmp_sparse() -> bool {
    let x = Simd::from_array([1u64, 2, 4, 8]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_sparse() -> bool {
    let x = Simd::from_array([1u64, 2, 4, 8]);
    is_empty_or(x)
}

#[divan::bench]
fn cmp_first_lane_only() -> bool {
    let x = Simd::from_array([u64::MAX, 0, 0, 0]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_first_lane_only() -> bool {
    let x = Simd::from_array([u64::MAX, 0, 0, 0]);
    is_empty_or(x)
}

#[divan::bench]
fn cmp_last_lane_only() -> bool {
    let x = Simd::from_array([0, 0, 0, u64::MAX]);
    is_empty_reduce(x)
}

#[divan::bench]
fn or_last_lane_only() -> bool {
    let x = Simd::from_array([0, 0, 0, u64::MAX]);
    is_empty_or(x)
}
