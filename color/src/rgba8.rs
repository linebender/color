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
    /// Returns the color as a packed value, with `r` as the most significant byte and
    /// `a` the least.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | self.a as u32
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
    /// Returns the color as a packed value, with `r` as the most significant byte and
    /// `a` the least.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | self.a as u32
    }
}

impl From<Rgba8> for PremulColor<Srgb> {
    fn from(value: Rgba8) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}
