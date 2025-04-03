// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Types for tracking [`DynamicColor`](crate::DynamicColor) state.

use crate::x11_colors;

/// Flags indicating [`DynamicColor`](crate::DynamicColor) state.
///
/// The "missing" flags indicate whether a specific color component is missing (either the three
/// color channels or the alpha channel).
///
/// The "named" flag represents whether the dynamic color was parsed from one of the named colors
/// in [CSS Color Module Level 4 § 6.1][css-named-colors] or named color space functions in [CSS
/// Color Module Level 4 § 4.1][css-named-color-spaces].
///
/// The latter is primarily useful for serializing to a CSS-compliant string format.
///
/// [css-named-colors]: https://www.w3.org/TR/css-color-4/#named-colors
/// [css-named-color-spaces]: https://www.w3.org/TR/css-color-4/#color-syntax
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Flags {
    /// A bitset of missing color components.
    missing: Missing,

    /// The named source a [`crate::DynamicColor`] was constructed from. Meanings:
    /// - 0 - not constructed from a named source;
    /// - 255 - constructed from a named color space function;
    /// - otherwise - the 1-based index into [`crate::x11_colors::NAMES`].
    name: u8,
}

// Ensure the amount of colors fits into the `Flags::name` packing.
#[cfg(test)]
const _: () = const {
    if x11_colors::NAMES.len() > 253 {
        panic!("There are more X11 color names than can be packed into Flags.");
    }
};

/// Missing color components, extracted from [`Flags`].
///
/// Some bitwise operations are implemented on this type, making certain manipulations more
/// ergonomic.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Missing(u8);

impl Flags {
    /// Construct flags with the given missing components.
    #[inline]
    pub const fn from_missing(missing: Missing) -> Self {
        Self { missing, name: 0 }
    }

    /// Set the missing components.
    #[inline]
    #[warn(
        clippy::missing_const_for_fn,
        reason = "can be made const with MSRV 1.83"
    )]
    pub fn set_missing(&mut self, missing: Missing) {
        self.missing = missing;
    }

    /// Returns the missing components from the flags.
    #[inline]
    pub const fn missing(self) -> Missing {
        self.missing
    }

    /// Set the flags to indicate the color was specified as one of the named colors. `name_ix` is
    /// the index into [`crate::x11_colors::NAMES`].
    pub(crate) fn set_named_color(&mut self, name_ix: usize) {
        debug_assert!(
            name_ix < x11_colors::NAMES.len(),
            "Expected an X11 color name index no larger than: {}. Got: {}.",
            x11_colors::NAMES.len(),
            name_ix
        );

        #[expect(
            clippy::cast_possible_truncation,
            reason = "name_ix is guaranteed to small enough by the above condition and by the test on the length of `x11_colors::NAMES`"
        )]
        {
            self.name = name_ix as u8 + 1;
        }
    }

    /// Set the flags to indicate the color was specified using one of the named color space
    /// functions.
    #[warn(
        clippy::missing_const_for_fn,
        reason = "can be made const with MSRV 1.83"
    )]
    pub(crate) fn set_named_color_space(&mut self) {
        self.name = 255;
    }

    /// Returns `true` if the flags indicate the color was generated from a named color or named
    /// color space function.
    #[inline]
    pub const fn named(self) -> bool {
        self.name != 0
    }

    /// If the color was constructed from a named color, returns that name.
    ///
    /// See also [`parse_color`][crate::parse_color].
    pub const fn color_name(self) -> Option<&'static str> {
        let name_ix = self.name;
        if name_ix == 0 || name_ix == 255 {
            None
        } else {
            Some(x11_colors::NAMES[name_ix as usize - 1])
        }
    }

    /// Discard the color name or color space name from the flags.
    #[inline]
    #[warn(
        clippy::missing_const_for_fn,
        reason = "can be made const with MSRV 1.83"
    )]
    pub fn discard_name(&mut self) {
        self.name = 0;
    }
}

impl core::fmt::Debug for Flags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Flags")
            .field("missing", &self.missing)
            .field("name", &self.name)
            .field("named", &self.named())
            .field("color_name", &self.color_name())
            .finish()
    }
}

impl Missing {
    /// The set containing a single component index.
    #[inline]
    pub const fn single(ix: usize) -> Self {
        debug_assert!(ix <= 3, "color component index must be 0, 1, 2 or 3");
        Self(1 << ix)
    }

    /// Returns `true` if the set contains the component index.
    #[inline]
    pub const fn contains(self, ix: usize) -> bool {
        (self.0 & Self::single(ix).0) != 0
    }

    /// Add a missing component index to the set.
    #[inline]
    #[warn(
        clippy::missing_const_for_fn,
        reason = "can be made const with MSRV 1.83"
    )]
    pub fn insert(&mut self, ix: usize) {
        self.0 |= Self::single(ix).0;
    }

    /// Returns `true` if the set contains no indices.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitAnd for Missing {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitOr for Missing {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::Not for Missing {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl core::fmt::Debug for Missing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Missing")
            .field(&format_args!("{:#010b}", self.0))
            .finish()
    }
}
