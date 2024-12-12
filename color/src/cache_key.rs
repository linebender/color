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
/// types containing colours.
///
/// See the [module level docs](self) for more information.
#[derive(Debug, Copy, Clone)]
pub struct CacheKey<T>(pub T);

// This module exists for these implementations:

// `BitEq` is an equivalence relation, just maybe not the one you'd expect.
impl<T: BitEq> Eq for CacheKey<T> {}
impl<T: BitEq> PartialEq for CacheKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bit_eq(other)
    }
}
// If we implement Eq, BitEq's implementation matches that of the hash.
impl<T: BitHash> Hash for CacheKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.bit_hash(state);
    }
}

/// An hash implementation for types normally wouldn't have one,
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

/// An equivalence relation for types normally wouldn't have
/// one, implemented using a bitwise comparison when needed.
///
/// See the docs on [`Eq`] for more information.
///
/// Useful for creating caches based on exact values.
/// See the [module level docs](self) for more information.
pub trait BitEq {
    /// Returns true if `self` is the same "value" as other.
    fn bit_eq(&self, other: &Self) -> bool;
    // Intentionally no bit_ne as would be added complexity for little gain
}

/// We already have an existing equivalence hash for these types, so just use that.
impl<T> BitHash for T
where
    T: Hash,
{
    fn bit_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

/// We already have an existing equivalence relation for these types, so just use that.
impl<T> BitEq for T
where
    T: PartialEq + Eq,
{
    fn bit_eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}
