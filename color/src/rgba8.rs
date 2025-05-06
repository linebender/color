// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{AlphaColor, PremulColor, Srgb};

/// A packed representation of sRGB colors.
///
/// Encoding sRGB with 8 bits per component is extremely common, as
/// it is efficient and convenient, even if limited in accuracy and
/// gamut.
///
/// This is not meant to be a general purpose color type and is
/// intended for use with [`AlphaColor::to_rgba8`] and [`OpaqueColor::to_rgba8`].
///
/// For a pre-multiplied packed representation, see [`PremulRgba8`].
///
/// [`AlphaColor::to_rgba8`]: crate::AlphaColor::to_rgba8
/// [`OpaqueColor::to_rgba8`]: crate::OpaqueColor::to_rgba8
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
    ///
    /// The color values will be in the order `[r, g, b, a]`.
    #[must_use]
    pub const fn to_u8_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert the `[u8; 4]` byte array into an `Rgba8` color.
    ///
    /// The color values must be given in the order `[r, g, b, a]`.
    #[must_use]
    pub const fn from_u8_array([r, g, b, a]: [u8; 4]) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the color as a little-endian packed value, with `r` the least significant byte and
    /// `a` the most significant.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        u32::from_ne_bytes(self.to_u8_array())
    }

    /// Interpret the little-endian packed value as a color, with `r` the least significant byte
    /// and `a` the most significant.
    #[must_use]
    pub const fn from_u32(packed_bytes: u32) -> Self {
        Self::from_u8_array(u32::to_ne_bytes(packed_bytes))
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
///
/// This is not meant to be a general purpose color type and is
/// intended for use with [`PremulColor::to_rgba8`].
///
/// For a non-pre-multiplied packed representation, see [`Rgba8`].
///
/// [`PremulColor::to_rgba8`]: crate::PremulColor::to_rgba8
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
    ///
    /// The color values will be in the order `[r, g, b, a]`.
    #[must_use]
    pub const fn to_u8_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert the `[u8; 4]` byte array into a `PremulRgba8` color.
    ///
    /// The color values must be given in the order `[r, g, b, a]`.
    #[must_use]
    pub const fn from_u8_array([r, g, b, a]: [u8; 4]) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the color as a little-endian packed value, with `r` the least significant byte and
    /// `a` the most significant.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        u32::from_ne_bytes(self.to_u8_array())
    }

    /// Interpret the little-endian packed value as a color, with `r` the least significant byte
    /// and `a` the most significant.
    #[must_use]
    pub const fn from_u32(packed_bytes: u32) -> Self {
        Self::from_u8_array(u32::to_ne_bytes(packed_bytes))
    }
}

impl From<PremulRgba8> for PremulColor<Srgb> {
    fn from(value: PremulRgba8) -> Self {
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
        assert_eq!(0x04030201_u32.to_le(), c.to_u32());

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(0xffccbbaa_u32.to_le(), p.to_u32());
    }

    #[test]
    fn from_u32() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!(Rgba8::from_u32(0x04030201_u32.to_le()), c);

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(PremulRgba8::from_u32(0xffccbbaa_u32.to_le()), p);
    }

    #[test]
    fn to_u8_array() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!([1, 2, 3, 4], c.to_u8_array());

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!([0xaa, 0xbb, 0xcc, 0xff], p.to_u8_array());
    }

    #[test]
    fn from_u8_array() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!(Rgba8::from_u8_array([1, 2, 3, 4]), c);

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(PremulRgba8::from_u8_array([0xaa, 0xbb, 0xcc, 0xff]), p);
    }

    #[test]
    #[cfg(feature = "bytemuck")]
    fn bytemuck_to_u32() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!(c.to_u32(), bytemuck::cast(c));

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(p.to_u32(), bytemuck::cast(p));
    }

    #[test]
    #[cfg(feature = "bytemuck")]
    fn bytemuck_from_u32() {
        let c = 0x04030201_u32.to_le();
        assert_eq!(Rgba8::from_u32(c), bytemuck::cast(c));

        let p = 0xffccbbaa_u32.to_le();
        assert_eq!(PremulRgba8::from_u32(p), bytemuck::cast(p));
    }

    #[test]
    #[cfg(feature = "bytemuck")]
    fn bytemuck_to_u8_array() {
        let c = Rgba8 {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        assert_eq!(c.to_u8_array(), bytemuck::cast::<_, [u8; 4]>(c));
        assert_eq!(c.to_u8_array(), bytemuck::cast::<_, [u8; 4]>(c.to_u32()));

        let p = PremulRgba8 {
            r: 0xaa,
            g: 0xbb,
            b: 0xcc,
            a: 0xff,
        };
        assert_eq!(p.to_u8_array(), bytemuck::cast::<_, [u8; 4]>(p));
        assert_eq!(p.to_u8_array(), bytemuck::cast::<_, [u8; 4]>(p.to_u32()));
    }

    #[test]
    #[cfg(feature = "bytemuck")]
    fn bytemuck_from_u8_array() {
        let c = [1, 2, 3, 4];
        assert_eq!(Rgba8::from_u8_array(c), bytemuck::cast(c));

        let p = [0xaa, 0xbb, 0xcc, 0xff];
        assert_eq!(PremulRgba8::from_u8_array(p), bytemuck::cast(p));
    }
}
