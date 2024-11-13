// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A simple bitset.

use crate::x11_colors;

/// Flags indicating [`DynamicColor`](crate::DynamicColor) state.
///
/// This tracks missing color components of a `DynamicColor` and details of how a `DynamicColor`
/// was constructed.
///
/// The "missing" flags indicate whether a specific color component is missing. The "named" flag
/// represents whether the dynamic color was generated from one of the named colors in [CSS Color
/// Module Level 4 § 6.1][css-named-colors] or named color space functions in [CSS Color Module
/// Level 4 § 4.1][css-named-color-spaces].
///
/// The latter is primarily useful for serializing.
///
/// [css-named-colors]: https://www.w3.org/TR/css-color-4/#named-colors
/// [css-named-color-spaces]: https://www.w3.org/TR/css-color-4/#color-syntax
//
// The flags are tracked with 16 bits. The first three are for missing components, the fourth
// indicates whether the color was generated from a named color or named color space function. The
// remaining bits indicate the named color.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Flags(u16);

/// Missing color components, extracted from [`Flags`]. Some bitwise operations are implemented on
/// this type, making certain manipulations more ergonomic.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Missing(u8);

impl Flags {
    /// Construct flags indicating the given color component is missing. The component index must
    /// be 0, 1, 2, or 3.
    pub const fn from_single_missing(ix: usize) -> Self {
        debug_assert!(ix <= 3, "color component index must be 0, 1, 2 or 3");
        Self(1 << ix)
    }

    /// Construct flags with the given missing components.
    pub const fn from_missing(missing: Missing) -> Self {
        Self(missing.0 as u16)
    }

    /// Construct flags indicating the color was generated from one of the named colors.
    pub(crate) fn set_named_color(&mut self, name_ix: u16) {
        let missing = self.0 & 0b1111;
        self.0 = missing | 1 << 4 | (name_ix + 1) << 5;
    }

    /// Construct flags indicating the color was generated from one of the named color space
    /// functions.
    pub(crate) fn set_named_color_space(&mut self) {
        let missing = self.0 & 0b1111;
        self.0 = missing | 1 << 4;
    }

    /// Set the given component as missing.
    pub fn set_missing(&mut self, ix: usize) {
        self.0 |= Self::from_single_missing(ix).0;
    }

    /// Extract the missing components from the flags.
    #[inline]
    pub fn extract_missing(self) -> Missing {
        Missing((self.0 & 0b1111) as u8)
    }

    /// Returns `true` if the flags indicate the given color component is missing. The component
    /// index must be 0, 1 or 2.
    #[inline]
    pub fn missing(self, ix: usize) -> bool {
        self.extract_missing().contains(ix)
    }

    /// Returns `true` if at least one component is missing.
    #[inline]
    pub fn has_missing(self) -> bool {
        !self.extract_missing().is_empty()
    }

    /// Returns `true` if the flags indicate the color was generated from a named color or named
    /// color space function.
    pub fn named(self) -> bool {
        self.0 & 1 << 4 != 0
    }

    /// If the color was constructed from a named color, returns that name.
    ///
    /// See also [`parse_color`][crate::parse_color].
    pub fn color_name(self) -> Option<&'static str> {
        let name_ix = self.0 >> 5;
        if name_ix == 0 {
            None
        } else {
            Some(x11_colors::NAMES[name_ix as usize - 1])
        }
    }
}

impl Missing {
    /// Returns `true` if the set contains the component index.
    #[inline]
    pub fn contains(self, ix: usize) -> bool {
        (self.0 & (1 << ix)) != 0
    }

    /// Adds a component index to the set.
    #[inline]
    pub fn insert(&mut self, ix: usize) {
        self.0 |= 1 << ix;
    }

    /// The set containing a single component index.
    #[inline]
    pub fn single(ix: usize) -> Self {
        Self(1 << ix)
    }

    /// Returns `true` if the set contains no indices.
    #[inline]
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
