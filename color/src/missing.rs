// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A simple bitset.

/// A simple bitset for representing missing components.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Missing(u8);

impl Missing {
    /// Returns `true` if the set contains the component index.
    pub fn contains(self, ix: usize) -> bool {
        (self.0 & (1 << ix)) != 0
    }

    /// Adds a component index to the set.
    pub fn insert(&mut self, ix: usize) {
        self.0 |= 1 << ix;
    }

    /// The set containing a single component index.
    pub fn single(ix: usize) -> Self {
        Self(1 << ix)
    }

    /// Returns `true` if the set contains no indices.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitAnd for Missing {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitOr for Missing {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::Not for Missing {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
