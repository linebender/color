// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A simple bitset.

use crate::x11_colors;

/// Flags indicating [`DynamicColor`](crate::DynamicColor) state.
///
/// This tracks missing color components of a `DynamicColor` and details of how a `DynamicColor`
/// was constructed.
///
/// The "missing" flags indicate whether a specific color component is missing (either the three
/// color channels or the alpha channel). The "named" flag represents whether the dynamic color was
/// generated from one of the named colors in [CSS Color Module Level 4 § 6.1][css-named-colors] or
/// named color space functions in [CSS Color Module Level 4 § 4.1][css-named-color-spaces].
///
/// The latter is primarily useful for serializing.
///
/// [css-named-colors]: https://www.w3.org/TR/css-color-4/#named-colors
/// [css-named-color-spaces]: https://www.w3.org/TR/css-color-4/#color-syntax
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// A bitset of missing color components.
    missing: u8,

    /// The named source a [`crate::DynamicColor`] was constructed from. Meanings:
    /// - 0 - not constructed from a named source;
    /// - 255 - constructed from a named color space function;
    /// - otherwise - the 1-based index into [`crate::x11_colors::NAMES`].
    name: u8,
}

/// Missing color components, extracted from [`Flags`]. Some bitwise operations are implemented on
/// this type, making certain manipulations more ergonomic.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Missing(u8);

impl Flags {
    /// Construct flags indicating the given color component is missing. The component index must
    /// be 0, 1, 2, or 3.
    pub const fn from_single_missing(ix: usize) -> Self {
        debug_assert!(ix <= 3, "color component index must be 0, 1, 2 or 3");
        Flags {
            missing: 1 << ix,
            name: 0,
        }
    }

    /// Construct flags with the given missing components.
    pub const fn from_missing(missing: Missing) -> Self {
        Flags {
            missing: missing.0,
            name: 0,
        }
    }

    /// Construct flags indicating the color was generated from one of the named colors.
    pub(crate) fn set_named_color(&mut self, name_ix: usize) {
        debug_assert!(name_ix < x11_colors::NAMES.len());
        debug_assert!(x11_colors::NAMES.len() <= 253);

        self.name = name_ix as u8 + 1;
    }

    /// Construct flags indicating the color was generated from one of the named color space
    /// functions.
    pub(crate) fn set_named_color_space(&mut self) {
        self.name = 255;
    }

    /// Set the given component as missing.
    pub fn set_missing(&mut self, ix: usize) {
        self.missing = Self::from_single_missing(ix).missing;
    }

    /// Extract the missing components from the flags.
    #[inline]
    pub fn extract_missing(self) -> Missing {
        Missing(self.missing)
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
        self.name != 0
    }

    /// If the color was constructed from a named color, returns that name.
    ///
    /// See also [`parse_color`][crate::parse_color].
    pub fn color_name(self) -> Option<&'static str> {
        let name_ix = self.name;
        if name_ix == 0 || name_ix == 255 {
            None
        } else {
            Some(x11_colors::NAMES[name_ix as usize - 1])
        }
    }
}

impl core::fmt::Debug for Flags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Flags")
            .field(
                "data",
                &format_args!("{:#018b}", (self.missing as u16) << 8 + self.name as u16),
            )
            .field(
                "missing",
                &[
                    self.missing(0),
                    self.missing(1),
                    self.missing(2),
                    self.missing(3),
                ],
            )
            .field("named", &self.named())
            .field("color_name", &self.color_name())
            .finish()
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

impl core::fmt::Debug for Missing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Missing")
            .field("data", &format_args!("{:#010b}", self.0))
            .field(
                "missing",
                &[
                    self.contains(0),
                    self.contains(1),
                    self.contains(2),
                    self.contains(3),
                ],
            )
            .finish()
    }
}
