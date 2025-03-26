// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The color space tag enum.

use crate::{
    A98Rgb, Aces2065_1, AcesCg, Chromaticity, ColorSpace, ColorSpaceLayout, DisplayP3, Hsl, Hwb,
    Lab, Lch, LinearSrgb, Missing, Oklab, Oklch, ProphotoRgb, Rec2020, Srgb, XyzD50, XyzD65,
};

/// The color space tag for [dynamic colors].
///
/// This represents a fixed set of known color spaces. The set contains all
/// color spaces in the CSS Color 4 spec and includes some other color spaces
/// useful for computer graphics.
///
/// The integer values of these variants can change in breaking releases.
///
/// [dynamic colors]: crate::DynamicColor
//
// Note: when adding an RGB-like color space, add to `same_analogous`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
#[repr(u8)]
pub enum ColorSpaceTag {
    /// The [`Srgb`] color space.
    Srgb = 0,
    /// The [`LinearSrgb`] color space.
    LinearSrgb = 1,
    /// The [`Lab`] color space.
    Lab = 2,
    /// The [`Lch`] color space.
    Lch = 3,
    /// The [`Hsl`] color space.
    Hsl = 4,
    /// The [`Hwb`] color space.
    Hwb = 5,
    /// The [`Oklab`] color space.
    Oklab = 6,
    /// The [`Oklch`] color space.
    Oklch = 7,
    /// The [`DisplayP3`] color space.
    DisplayP3 = 8,
    /// The [`A98Rgb`] color space.
    A98Rgb = 9,
    /// The [`ProphotoRgb`] color space.
    ProphotoRgb = 10,
    /// The [`Rec2020`] color space.
    Rec2020 = 11,
    /// The [`Aces2065_1`] color space.
    Aces2065_1 = 15,
    /// The [`AcesCg`] color space.
    AcesCg = 12,
    /// The [`XyzD50`] color space.
    XyzD50 = 13,
    /// The [`XyzD65`] color space.
    XyzD65 = 14,
    // NOTICE: If a new value is added, be sure to modify `MAX_VALUE` in the bytemuck impl. Also
    // note the variants' integer values are not necessarily in order, allowing newly added color
    // space tags to be grouped with related color spaces.
}

impl ColorSpaceTag {
    pub(crate) fn layout(self) -> ColorSpaceLayout {
        match self {
            Self::Lch | Self::Oklch => ColorSpaceLayout::HueThird,
            Self::Hsl | Self::Hwb => ColorSpaceLayout::HueFirst,
            _ => ColorSpaceLayout::Rectangular,
        }
    }

    /// Whether all components of the two color spaces are analogous. See also
    /// Section 12.2 of CSS Color 4, defining which components are analogous:
    /// <https://www.w3.org/TR/2024/CRD-css-color-4-20240213/#interpolation-missing>.
    ///
    /// Note: if color spaces are the same, then they're also analogous, but
    /// in that case we wouldn't do the conversion, so this function is not
    /// guaranteed to return the correct answer in those cases.
    pub(crate) fn same_analogous(self, other: Self) -> bool {
        use ColorSpaceTag::*;
        matches!(
            (self, other),
            (
                Srgb | LinearSrgb
                    | DisplayP3
                    | A98Rgb
                    | ProphotoRgb
                    | Rec2020
                    | Aces2065_1
                    | AcesCg
                    | XyzD50
                    | XyzD65,
                Srgb | LinearSrgb
                    | DisplayP3
                    | A98Rgb
                    | ProphotoRgb
                    | Rec2020
                    | Aces2065_1
                    | AcesCg
                    | XyzD50
                    | XyzD65
            ) | (Lab | Oklab, Lab | Oklab)
                | (Lch | Oklch, Lch | Oklch)
        )
    }

    pub(crate) fn l_missing(self, missing: Missing) -> bool {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch => missing.contains(0),
            Hsl => missing.contains(2),
            _ => false,
        }
    }

    pub(crate) fn set_l_missing(self, missing: &mut Missing, components: &mut [f32; 4]) {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch => {
                missing.insert(0);
                components[0] = 0.0;
            }
            Hsl => {
                missing.insert(2);
                components[2] = 0.0;
            }
            _ => (),
        }
    }

    pub(crate) fn c_missing(self, missing: Missing) -> bool {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch | Hsl => missing.contains(1),
            _ => false,
        }
    }

    pub(crate) fn set_c_missing(self, missing: &mut Missing, components: &mut [f32; 4]) {
        use ColorSpaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch | Hsl => {
                missing.insert(1);
                components[1] = 0.0;
            }
            _ => (),
        }
    }

    pub(crate) fn h_missing(self, missing: Missing) -> bool {
        self.layout()
            .hue_channel()
            .is_some_and(|ix| missing.contains(ix))
    }

    pub(crate) fn set_h_missing(self, missing: &mut Missing, components: &mut [f32; 4]) {
        if let Some(ix) = self.layout().hue_channel() {
            missing.insert(ix);
            components[ix] = 0.0;
        }
    }

    /// Convert an opaque color from linear sRGB.
    ///
    /// This is the tagged counterpart of [`ColorSpace::from_linear_srgb`].
    pub fn from_linear_srgb(self, rgb: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::from_linear_srgb(rgb),
            Self::LinearSrgb => rgb,
            Self::Lab => Lab::from_linear_srgb(rgb),
            Self::Lch => Lch::from_linear_srgb(rgb),
            Self::Oklab => Oklab::from_linear_srgb(rgb),
            Self::Oklch => Oklch::from_linear_srgb(rgb),
            Self::DisplayP3 => DisplayP3::from_linear_srgb(rgb),
            Self::A98Rgb => A98Rgb::from_linear_srgb(rgb),
            Self::ProphotoRgb => ProphotoRgb::from_linear_srgb(rgb),
            Self::Rec2020 => Rec2020::from_linear_srgb(rgb),
            Self::Aces2065_1 => Aces2065_1::from_linear_srgb(rgb),
            Self::AcesCg => AcesCg::from_linear_srgb(rgb),
            Self::XyzD50 => XyzD50::from_linear_srgb(rgb),
            Self::XyzD65 => XyzD65::from_linear_srgb(rgb),
            Self::Hsl => Hsl::from_linear_srgb(rgb),
            Self::Hwb => Hwb::from_linear_srgb(rgb),
        }
    }

    /// Convert an opaque color to linear sRGB.
    ///
    /// This is the tagged counterpart of [`ColorSpace::to_linear_srgb`].
    pub fn to_linear_srgb(self, src: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::to_linear_srgb(src),
            Self::LinearSrgb => src,
            Self::Lab => Lab::to_linear_srgb(src),
            Self::Lch => Lch::to_linear_srgb(src),
            Self::Oklab => Oklab::to_linear_srgb(src),
            Self::Oklch => Oklch::to_linear_srgb(src),
            Self::DisplayP3 => DisplayP3::to_linear_srgb(src),
            Self::A98Rgb => A98Rgb::to_linear_srgb(src),
            Self::ProphotoRgb => ProphotoRgb::to_linear_srgb(src),
            Self::Rec2020 => Rec2020::to_linear_srgb(src),
            Self::Aces2065_1 => Aces2065_1::to_linear_srgb(src),
            Self::AcesCg => AcesCg::to_linear_srgb(src),
            Self::XyzD50 => XyzD50::to_linear_srgb(src),
            Self::XyzD65 => XyzD65::to_linear_srgb(src),
            Self::Hsl => Hsl::to_linear_srgb(src),
            Self::Hwb => Hwb::to_linear_srgb(src),
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
            (Self::Srgb, Self::Hsl) => Srgb::convert::<Hsl>(src),
            (Self::Hsl, Self::Srgb) => Hsl::convert::<Srgb>(src),
            (Self::Srgb, Self::Hwb) => Srgb::convert::<Hwb>(src),
            (Self::Hwb, Self::Srgb) => Hwb::convert::<Srgb>(src),
            (Self::Hsl, Self::Hwb) => Hsl::convert::<Hwb>(src),
            (Self::Hwb, Self::Hsl) => Hwb::convert::<Hsl>(src),
            _ => target.from_linear_srgb(self.to_linear_srgb(src)),
        }
    }

    /// Convert an opaque color from linear sRGB, without chromatic adaptation.
    ///
    /// For most use-cases you should consider using the chromatically-adapting
    /// [`ColorSpaceTag::from_linear_srgb`] instead.
    ///
    /// This is the tagged counterpart of [`ColorSpace::from_linear_srgb_absolute`].
    pub fn from_linear_srgb_absolute(self, rgb: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::from_linear_srgb_absolute(rgb),
            Self::LinearSrgb => rgb,
            Self::Lab => Lab::from_linear_srgb_absolute(rgb),
            Self::Lch => Lch::from_linear_srgb_absolute(rgb),
            Self::Oklab => Oklab::from_linear_srgb_absolute(rgb),
            Self::Oklch => Oklch::from_linear_srgb_absolute(rgb),
            Self::DisplayP3 => DisplayP3::from_linear_srgb_absolute(rgb),
            Self::A98Rgb => A98Rgb::from_linear_srgb_absolute(rgb),
            Self::ProphotoRgb => ProphotoRgb::from_linear_srgb_absolute(rgb),
            Self::Rec2020 => Rec2020::from_linear_srgb_absolute(rgb),
            Self::Aces2065_1 => Aces2065_1::from_linear_srgb_absolute(rgb),
            Self::AcesCg => AcesCg::from_linear_srgb_absolute(rgb),
            Self::XyzD50 => XyzD50::from_linear_srgb_absolute(rgb),
            Self::XyzD65 => XyzD65::from_linear_srgb_absolute(rgb),
            Self::Hsl => Hsl::from_linear_srgb_absolute(rgb),
            Self::Hwb => Hwb::from_linear_srgb_absolute(rgb),
        }
    }

    /// Convert an opaque color to linear sRGB, without chromatic adaptation.
    ///
    /// For most use-cases you should consider using the chromatically-adapting
    /// [`ColorSpaceTag::to_linear_srgb`] instead.
    ///
    /// This is the tagged counterpart of [`ColorSpace::to_linear_srgb_absolute`].
    pub fn to_linear_srgb_absolute(self, src: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::to_linear_srgb_absolute(src),
            Self::LinearSrgb => src,
            Self::Lab => Lab::to_linear_srgb_absolute(src),
            Self::Lch => Lch::to_linear_srgb_absolute(src),
            Self::Oklab => Oklab::to_linear_srgb_absolute(src),
            Self::Oklch => Oklch::to_linear_srgb_absolute(src),
            Self::DisplayP3 => DisplayP3::to_linear_srgb_absolute(src),
            Self::A98Rgb => A98Rgb::to_linear_srgb_absolute(src),
            Self::ProphotoRgb => ProphotoRgb::to_linear_srgb_absolute(src),
            Self::Rec2020 => Rec2020::to_linear_srgb_absolute(src),
            Self::Aces2065_1 => Aces2065_1::to_linear_srgb_absolute(src),
            Self::AcesCg => AcesCg::to_linear_srgb_absolute(src),
            Self::XyzD50 => XyzD50::to_linear_srgb_absolute(src),
            Self::XyzD65 => XyzD65::to_linear_srgb_absolute(src),
            Self::Hsl => Hsl::to_linear_srgb_absolute(src),
            Self::Hwb => Hwb::to_linear_srgb_absolute(src),
        }
    }

    /// Convert the color components into the target color space, without chromatic adaptation.
    ///
    /// For most use-cases you should consider using the chromatically-adapting
    /// [`ColorSpaceTag::convert`] instead.
    ///
    /// This is the tagged counterpart of [`ColorSpace::convert_absolute`]. See the documentation
    /// on [`ColorSpace::convert_absolute`] for more information.
    pub fn convert_absolute(self, target: Self, src: [f32; 3]) -> [f32; 3] {
        match (self, target) {
            _ if self == target => src,
            (Self::Oklab, Self::Oklch) | (Self::Lab, Self::Lch) => {
                Oklab::convert_absolute::<Oklch>(src)
            }
            (Self::Oklch, Self::Oklab) | (Self::Lch, Self::Lab) => {
                Oklch::convert_absolute::<Oklab>(src)
            }
            (Self::Srgb, Self::Hsl) => Srgb::convert_absolute::<Hsl>(src),
            (Self::Hsl, Self::Srgb) => Hsl::convert_absolute::<Srgb>(src),
            (Self::Srgb, Self::Hwb) => Srgb::convert_absolute::<Hwb>(src),
            (Self::Hwb, Self::Srgb) => Hwb::convert_absolute::<Srgb>(src),
            (Self::Hsl, Self::Hwb) => Hsl::convert_absolute::<Hwb>(src),
            (Self::Hwb, Self::Hsl) => Hwb::convert_absolute::<Hsl>(src),
            _ => target.from_linear_srgb_absolute(self.to_linear_srgb_absolute(src)),
        }
    }

    /// Chromatically adapt the color between the given white point chromaticities.
    ///
    /// This is the tagged counterpart of [`ColorSpace::chromatically_adapt`].
    ///
    /// The color is assumed to be under a reference white point of `from` and is chromatically
    /// adapted to the given white point `to`. The linear Bradford transform is used to perform the
    /// chromatic adaptation.
    pub fn chromatically_adapt(
        self,
        src: [f32; 3],
        from: Chromaticity,
        to: Chromaticity,
    ) -> [f32; 3] {
        match self {
            Self::Srgb => Srgb::chromatically_adapt(src, from, to),
            Self::LinearSrgb => LinearSrgb::chromatically_adapt(src, from, to),
            Self::Lab => Lab::chromatically_adapt(src, from, to),
            Self::Lch => Lch::chromatically_adapt(src, from, to),
            Self::Oklab => Oklab::chromatically_adapt(src, from, to),
            Self::Oklch => Oklch::chromatically_adapt(src, from, to),
            Self::DisplayP3 => DisplayP3::chromatically_adapt(src, from, to),
            Self::A98Rgb => A98Rgb::chromatically_adapt(src, from, to),
            Self::ProphotoRgb => ProphotoRgb::chromatically_adapt(src, from, to),
            Self::Rec2020 => Rec2020::chromatically_adapt(src, from, to),
            Self::Aces2065_1 => Aces2065_1::chromatically_adapt(src, from, to),
            Self::AcesCg => AcesCg::chromatically_adapt(src, from, to),
            Self::XyzD50 => XyzD50::chromatically_adapt(src, from, to),
            Self::XyzD65 => XyzD65::chromatically_adapt(src, from, to),
            Self::Hsl => Hsl::chromatically_adapt(src, from, to),
            Self::Hwb => Hwb::chromatically_adapt(src, from, to),
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
            Self::Lab => Lab::clip(src),
            Self::Lch => Lch::clip(src),
            Self::Oklab => Oklab::clip(src),
            Self::Oklch => Oklch::clip(src),
            Self::DisplayP3 => DisplayP3::clip(src),
            Self::A98Rgb => A98Rgb::clip(src),
            Self::ProphotoRgb => ProphotoRgb::clip(src),
            Self::Rec2020 => Rec2020::clip(src),
            Self::Aces2065_1 => Aces2065_1::clip(src),
            Self::AcesCg => AcesCg::clip(src),
            Self::XyzD50 => XyzD50::clip(src),
            Self::XyzD65 => XyzD65::clip(src),
            Self::Hsl => Hsl::clip(src),
            Self::Hwb => Hwb::clip(src),
        }
    }
}
