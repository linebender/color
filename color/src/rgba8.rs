// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{AlphaColor, PremulColor, Srgb};

/// A packed representation of sRGB colors.
///
/// Encoding sRGB with 8 bits per component is extremely common, as
/// it is efficient and convenient, even if limited in accuracy and
/// gamut.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct Rgba8 {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    ///
    /// Alpha is interpreted as separated alpha.
    pub a: u8,
}

impl Rgba8 {
    /// Returns the color as a `[u8; 4]`.
    #[must_use]
    pub const fn to_u8_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Returns the color as a little endian packed value, with `a` as the
    /// most significant byte and `r` the least.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        u32::from_le_bytes(self.to_u8_array())
    }
}

impl From<Rgba8> for AlphaColor<Srgb> {
    fn from(value: Rgba8) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}

/// A packed representation of pre-multiplied sRGB colors.
///
/// Encoding sRGB with 8 bits per component is extremely common, as
/// it is efficient and convenient, even if limited in accuracy and
/// gamut.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(C)]
pub struct PremulRgba8 {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    pub a: u8,
}

impl PremulRgba8 {
    /// Returns the color as a `[u8; 4]`.
    #[must_use]
    pub const fn to_u8_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Returns the color as a little endian packed value, with `a` as the
    /// most significant byte and `r` the least.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        u32::from_le_bytes(self.to_u8_array())
    }
}

impl From<Rgba8> for PremulColor<Srgb> {
    fn from(value: Rgba8) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}

#[cfg(test)]
mod tests {
    use super::{PremulRgba8, Rgba8};

    #[test]
    fn to_u32() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!(0x04030201_u32, c.to_u32());

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(0xffccbbaa_u32, p.to_u32());
    }
}
