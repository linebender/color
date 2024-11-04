// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A simple bitset.

/// A simple bitset, for representing missing components.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bitset(u8);

impl Bitset {
    pub fn contains(self, ix: usize) -> bool {
        (self.0 & (1 << ix)) != 0
    }

    pub fn set(&mut self, ix: usize) {
        self.0 |= 1 << ix;
    }

    pub fn single(ix: usize) -> Self {
        Self(1 << ix)
    }

    pub fn any(self) -> bool {
        self.0 != 0
    }
}

impl core::ops::BitAnd for Bitset {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitOr for Bitset {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::Not for Bitset {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
