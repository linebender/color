// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Colors with runtime choice of color space.

use crate::{
    color::{add_alpha, split_alpha},
    AlphaColor, Bitset, ColorSpace, ColorSpaceLayout, DisplayP3, LinearSrgb, Oklab, Oklch, Srgb,
    XyzD65,
};

/// The color space tag for tagged colors.
///
/// This represents a fixed set of known color spaces. The set is
/// based on the CSS Color 4 spec, but might also extend to a small
/// set of color spaces used in 3D graphics.
///
/// Note: this has some tags not yet implemented.
///
/// Note: when adding an RGB-like color space, add to `same_analogous`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ColorSpaceTag {
    /// The [`Srgb`] color space.
    Srgb,
    /// The [`LinearSrgb`] color space.
    LinearSrgb,
    // TODO: link
    /// The `Lab` color space.
    Lab,
    // TODO: link
    /// The `Lch` color space.
    Lch,
    // TODO: link
    /// The `Hsl` color space.
    Hsl,
    // TODO: link
    /// The `Hsl` color space.
    Hwb,
    /// The [`Oklab`] color space.
    Oklab,
    /// The [`Oklch`] color space.
    Oklch,
    /// The [`DisplayP3`] color space.
    DisplayP3,
    /// The [`XyzD65`] color space.
    XyzD65,
}

/// A color with a runtime color space tag. This type will likely get merged with
/// [`CssColor`][crate::css::CssColor].
#[derive(Clone, Copy, Debug)]
pub struct TaggedColor {
    pub cs: ColorSpaceTag,
    pub components: [f32; 4],
}

impl ColorSpaceTag {
    pub(crate) fn layout(self) -> ColorSpaceLayout {
        match self {
            Self::Lch | Self::Oklch => ColorSpaceLayout::HueThird,
            Self::Hsl | Self::Hwb => ColorSpaceLayout::HueFirst,
            _ => ColorSpaceLayout::Rectangular,
        }
    }

    // Note: if color spaces are the same, then they're also analogous, but
    // in that case we wouldn't do the conversion, so this function is not
    // guaranteed to return the correct answer in those cases.
    pub(crate) fn same_analogous(self, other: Self) -> bool {
        use ColorSpaceTag::*;
        matches!(
            (self, other),
            (
                Srgb | LinearSrgb | DisplayP3 | XyzD65,
                Srgb | LinearSrgb | DisplayP3 | XyzD65
            ) | (Lab | Oklab, Lab | Oklab)
                | (Lch | Oklch, Lch | Oklch)
        )
    }

    pub(crate) fn l_missing(self, missing: Bitset) -> bool {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch => missing.contains(0),
            Hsl => missing.contains(2),
            _ => false,
        }
    }

    pub(crate) fn set_l_missing(self, missing: &mut Bitset, components: &mut [f32; 4]) {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch => {
                missing.set(0);
                components[0] = 0.0;
            }
            Hsl => {
                missing.set(2);
                components[2] = 0.0;
            }
            _ => (),
        }
    }

    pub(crate) fn c_missing(self, missing: Bitset) -> bool {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch | Hsl => missing.contains(1),
            _ => false,
        }
    }

    pub(crate) fn set_c_missing(self, missing: &mut Bitset, components: &mut [f32; 4]) {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch | Hsl => {
                missing.set(1);
                components[1] = 0.0;
            }
            _ => (),
        }
    }

    pub(crate) fn h_missing(self, missing: Bitset) -> bool {
        self.layout()
            .hue_channel()
            .is_some_and(|ix| missing.contains(ix))
    }

    pub(crate) fn set_h_missing(self, missing: &mut Bitset, components: &mut [f32; 4]) {
        if let Some(ix) = self.layout().hue_channel() {
            missing.set(ix);
            components[ix] = 0.0;
        }
    }

    /// Convert an opaque color from linear sRGB.
    ///
    /// This is the tagged counterpart of [`ColorSpace::to_linear_srgb`].
    pub fn from_linear_srgb(self, rgb: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::from_linear_srgb(rgb),
            Self::LinearSrgb => rgb,
            Self::Oklab => Oklab::from_linear_srgb(rgb),
            Self::Oklch => Oklch::from_linear_srgb(rgb),
            Self::DisplayP3 => DisplayP3::from_linear_srgb(rgb),
            Self::XyzD65 => XyzD65::from_linear_srgb(rgb),
            _ => todo!(),
        }
    }

    /// Convert an opaque color to linear sRGB.
    ///
    /// This is the tagged counterpart of [`ColorSpace::to_linear_srgb`].
    pub fn to_linear_srgb(self, src: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::to_linear_srgb(src),
            Self::LinearSrgb => src,
            Self::Oklab => Oklab::to_linear_srgb(src),
            Self::Oklch => Oklch::to_linear_srgb(src),
            Self::DisplayP3 => DisplayP3::to_linear_srgb(src),
            Self::XyzD65 => XyzD65::to_linear_srgb(src),
            _ => todo!(),
        }
    }

    /// Convert the color components into the target color space.
    ///
    /// This is the tagged counterpart of [`ColorSpace::convert`].
    pub fn convert(self, target: Self, src: [f32; 3]) -> [f32; 3] {
        match (self, target) {
            _ if self == target => src,
            (Self::Oklab, Self::Oklch) | (Self::Lab, Self::Lch) => Oklab::convert::<Oklch>(src),
            (Self::Oklch, Self::Oklab) | (Self::Lch, Self::Lab) => Oklch::convert::<Oklab>(src),
            _ => target.from_linear_srgb(self.to_linear_srgb(src)),
        }
    }

    /// Scale the chroma by the given amount.
    ///
    /// This is the tagged counterpart of [`ColorSpace::scale_chroma`].
    pub fn scale_chroma(self, src: [f32; 3], scale: f32) -> [f32; 3] {
        match self {
            Self::LinearSrgb => LinearSrgb::scale_chroma(src, scale),
            Self::Oklab | Self::Lab => Oklab::scale_chroma(src, scale),
            Self::Oklch | Self::Lch | Self::Hsl => Oklch::scale_chroma(src, scale),
            _ => {
                let rgb = self.to_linear_srgb(src);
                let scaled = LinearSrgb::scale_chroma(rgb, scale);
                self.from_linear_srgb(scaled)
            }
        }
    }

    /// Clip the color's components to fit within the natural gamut of the color space.
    ///
    /// See [`ColorSpace::clip`] for more details.
    pub fn clip(self, src: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::clip(src),
            Self::LinearSrgb => LinearSrgb::clip(src),
            Self::Oklab => Oklab::clip(src),
            Self::Oklch => Oklch::clip(src),
            Self::DisplayP3 => DisplayP3::clip(src),
            Self::XyzD65 => XyzD65::clip(src),
            _ => todo!(),
        }
    }
}

impl TaggedColor {
    #[must_use]
    pub fn from_alpha_color<CS: ColorSpace>(color: AlphaColor<CS>) -> Self {
        if let Some(cs) = CS::TAG {
            let components = color.components;
            Self { cs, components }
        } else {
            Self::from_alpha_color(color.convert::<LinearSrgb>())
        }
    }

    #[must_use]
    pub fn to_alpha_color<CS: ColorSpace>(&self) -> AlphaColor<CS> {
        if let Some(cs) = CS::TAG {
            AlphaColor::new(self.convert(cs).components)
        } else {
            self.to_alpha_color::<LinearSrgb>().convert()
        }
    }

    #[must_use]
    pub fn convert(self, cs: ColorSpaceTag) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        let components = add_alpha(self.cs.convert(cs, opaque), alpha);
        Self { components, cs }
    }
}
