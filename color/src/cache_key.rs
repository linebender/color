// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hashing and other caching utilities for Color types.
//!
//! In this crate, colors are implemented using `f32`.
//! This means that color types aren't `Hash` and `Eq` for good reasons:
//!
//! - Equality on these types is not reflexive (consider [NaN](f32::NAN)).
//! - Certain values have two representations (`-0` and `+0` are both zero).
//!
//! However, it is still useful to create caches which key off these values.
//! These are caches which don't have any semantic meaning, but instead
//! are used to avoid redundant calculations or storage.
//!
//! Color supports creating these caches by using [`CacheKey<T>`] as the key in
//! your cache.
//! `T` is the key type (i.e. a color) which you want to use as the key.
//! This `T` must implement both [`BitHash`] and [`BitEq`], which are
//! versions of the standard `Hash` and `Eq` traits which support implementations
//! for floating point numbers which might be unexpected outside of a caching context.

use core::hash::{Hash, Hasher};

/// A key usable in a hashmap to compare the bit representation
/// types containing colors.
///
/// See the [module level docs](self) for more information.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CacheKey<T>(pub T);

impl<T> CacheKey<T> {
    /// Create a new `CacheKey`.
    ///
    /// All fields are public, so the struct constructor can also be used.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

// This module exists for these implementations:

// `BitEq` is an equivalence relation, just maybe not the one you'd expect.
impl<T: BitEq> Eq for CacheKey<T> {}
impl<T: BitEq> PartialEq for CacheKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.bit_eq(&other.0)
    }
}
// If we implement Eq, BitEq's implementation matches that of the hash.
impl<T: BitHash> Hash for CacheKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.bit_hash(state);
    }
}

/// A hash implementation for types which normally wouldn't have one,
/// implemented using a hash of the bitwise equivalent types when needed.
///
/// If a type is `BitHash` and `BitEq`, then it is important that the following property holds:
///
/// ```text
/// k1 biteq k2 -> bithash(k1) == bithash(k2)
/// ```
///
/// See the docs on [`Hash`] for more information.
///
/// Useful for creating caches based on exact values.
/// See the [module level docs](self) for more information.
pub trait BitHash {
    /// Feeds this value into the given [`Hasher`].
    fn bit_hash<H: Hasher>(&self, state: &mut H);
    // Intentionally no hash_slice for simplicity.
}

impl BitHash for f32 {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}
impl<T: BitHash, const N: usize> BitHash for [T; N] {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self[..].bit_hash(state);
    }
}

impl<T: BitHash> BitHash for [T] {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        // In theory, we should use `write_length_prefix`, which is unstable:
        // https://github.com/rust-lang/rust/issues/96762
        // We could do that by (unsafely) casting to `[CacheKey<T>]`, then
        // using `Hash::hash` on the resulting slice.
        state.write_usize(self.len());
        for piece in self {
            piece.bit_hash(state);
        }
    }
}

impl<T: BitHash> BitHash for &T {
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        T::bit_hash(*self, state);
    }
}

// Don't BitHash tuples, not that important

/// An equivalence relation for types which normally wouldn't have
/// one, implemented using a bitwise comparison for floating point
/// values.
///
/// See the docs on [`Eq`] for more information.
///
/// Useful for creating caches based on exact values.
/// See the [module level docs](self) for more information.
pub trait BitEq {
    /// Returns `true` if `self` is equal to `other`.
    ///
    /// This need not use the semantically natural comparison operation
    /// for the type; indeed floating point types should implement this
    /// by comparing bit values.
    fn bit_eq(&self, other: &Self) -> bool;
    // Intentionally no bit_ne as would be added complexity for little gain
}

impl BitEq for f32 {
    fn bit_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl<T: BitEq, const N: usize> BitEq for [T; N] {
    fn bit_eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if !self[i].bit_eq(&other[i]) {
                return false;
            }
        }
        true
    }
}

impl<T: BitEq> BitEq for [T] {
    fn bit_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.iter().zip(other) {
            if !a.bit_eq(b) {
                return false;
            }
        }
        true
    }
}

impl<T: BitEq> BitEq for &T {
    fn bit_eq(&self, other: &Self) -> bool {
        T::bit_eq(*self, *other)
    }
}

// Don't BitEq tuples, not that important

// Ideally we'd also have these implementations, but they cause conflicts
// (in case std ever went mad and implemented Eq for f32, for example).
// impl<T: Hash> BitHash for T {...}
// impl<T: PartialEq + Eq> BitEq for T {...}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::CacheKey;
    use crate::{parse_color, DynamicColor};
    use std::collections::HashMap;

    #[test]
    fn bit_eq_hashmap() {
        let mut map: HashMap<CacheKey<f32>, i32> = HashMap::new();
        // The implementation for f32 is the base case.
        assert!(map.insert(CacheKey(0.0), 0).is_none());
        assert!(map.insert(CacheKey(-0.0), -1).is_none());
        assert!(map.insert(CacheKey(1.0), 1).is_none());
        assert!(map.insert(CacheKey(0.5), 5).is_none());

        assert_eq!(map.get(&CacheKey(1.0)).unwrap(), &1);
        assert_eq!(map.get(&CacheKey(0.0)).unwrap(), &0);
        assert_eq!(map.remove(&CacheKey(-0.0)).unwrap(), -1);
        assert!(!map.contains_key(&CacheKey(-0.0)));
        assert_eq!(map.get(&CacheKey(0.5)).unwrap(), &5);
    }
    #[test]
    fn bit_eq_color_hashmap() {
        let mut map: HashMap<CacheKey<DynamicColor>, i32> = HashMap::new();

        let red = parse_color("red").unwrap();
        let red2 = parse_color("red").unwrap();
        let other = parse_color("oklab(0.4 0.2 0.6)").unwrap();
        assert!(map.insert(CacheKey(red), 10).is_none());
        assert_eq!(map.insert(CacheKey(red2), 5).unwrap(), 10);
        assert!(map.insert(CacheKey(other), 15).is_none());
        assert_eq!(map.get(&CacheKey(other)).unwrap(), &15);
    }
}
