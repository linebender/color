// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Colors with runtime choice of colorspace.

use crate::{
    color::{add_alpha, split_alpha},
    AlphaColor, Bitset, Colorspace, ColorspaceLayout, DisplayP3, LinearSrgb, Oklab, Oklch, Srgb,
    XyzD65,
};

/// The colorspace tag for tagged colors.
///
/// This represents a fixed set of known colorspaces. The set is
/// based on the CSS Color 4 spec, but might also extend to a small
/// set of colorspaces used in 3D graphics.
///
/// Note: this has some tags not yet implemented.
///
/// Note: when adding an RGB-like colorspace, add to `same_analogous`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ColorspaceTag {
    Srgb,
    LinearSrgb,
    Lab,
    Lch,
    Hsl,
    Hwb,
    Oklab,
    Oklch,
    DisplayP3,
    XyzD65,
}

/// A color with a runtime colorspace tag. This type will likely get merged with
/// [`CssColor`].
#[derive(Clone, Copy, Debug)]
pub struct TaggedColor {
    pub cs: ColorspaceTag,
    pub components: [f32; 4],
}

impl ColorspaceTag {
    pub(crate) fn layout(self) -> ColorspaceLayout {
        match self {
            Self::Lch | Self::Oklch => ColorspaceLayout::HueThird,
            Self::Hsl | Self::Hwb => ColorspaceLayout::HueFirst,
            _ => ColorspaceLayout::Rectangular,
        }
    }

    // Note: if colorspaces are the same, then they're also analogous, but
    // in that case we wouldn't do the conversion, so this function is not
    // guaranteed to return the correct answer in those cases.
    pub(crate) fn same_analogous(self, other: Self) -> bool {
        use ColorspaceTag::*;
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
        use ColorspaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch => missing.contains(0),
            Hsl => missing.contains(2),
            _ => false,
        }
    }

    pub(crate) fn set_l_missing(self, missing: &mut Bitset, components: &mut [f32; 4]) {
        use ColorspaceTag::*;
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
        use ColorspaceTag::*;
        match self {
            Lab | Lch | Oklab | Oklch | Hsl => missing.contains(1),
            _ => false,
        }
    }

    pub(crate) fn set_c_missing(self, missing: &mut Bitset, components: &mut [f32; 4]) {
        use ColorspaceTag::*;
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

    /// Scale the chroma by the given amount.
    ///
    /// See [`Colorspace::scale_chroma`] for more details.
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
}

impl TaggedColor {
    pub fn from_linear_srgb(rgba: [f32; 4], cs: ColorspaceTag) -> Self {
        let (rgb, alpha) = split_alpha(rgba);
        let opaque = cs.from_linear_srgb(rgb);
        let components = add_alpha(opaque, alpha);
        Self { cs, components }
    }

    pub fn from_alpha_color<T: Colorspace>(color: AlphaColor<T>) -> Self {
        if let Some(cs) = T::CS_TAG {
            Self {
                cs,
                components: color.components,
            }
        } else {
            let components = color.convert::<LinearSrgb>().components;
            Self {
                cs: ColorspaceTag::LinearSrgb,
                components,
            }
        }
    }

    pub fn to_alpha_color<T: Colorspace>(&self) -> AlphaColor<T> {
        if T::CS_TAG == Some(self.cs) {
            AlphaColor::new(self.components)
        } else {
            let (opaque, alpha) = split_alpha(self.components);
            let rgb = self.cs.to_linear_srgb(opaque);
            let components = add_alpha(T::from_linear_srgb(rgb), alpha);
            AlphaColor::new(components)
        }
    }

    #[must_use]
    pub fn convert(self, cs: ColorspaceTag) -> Self {
        if self.cs == cs {
            self
        } else {
            let linear = self.to_alpha_color::<LinearSrgb>();
            Self::from_linear_srgb(linear.components, cs)
        }
    }
}
