// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{AlphaColor, Srgb};

/// A packed representation of sRGB colors.
///
/// Encoding sRGB with 8 bits per component is extremely common, as
/// it is efficient and convenient, even if limited in accuracy and
/// gamut.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl From<Rgba8> for AlphaColor<Srgb> {
    fn from(value: Rgba8) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}
