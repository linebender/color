// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Concrete types for colors.

use core::any::TypeId;
use core::marker::PhantomData;

use crate::{
    cache_key::{BitEq, BitHash},
    ColorSpace, ColorSpaceLayout, ColorSpaceTag, Oklab, Oklch, PremulRgba8, Rgba8, Srgb,
};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

/// An opaque color.
///
/// A color in a color space known at compile time, without transparency. Note
/// that "opaque" refers to the color, not the representation; the components
/// are publicly accessible.
///
/// Arithmetic traits are defined on this type, and operate component-wise. A
/// major motivation for including these is to enable weighted sums, including
/// for spline interpolation. For cylindrical color spaces, hue fixup should
/// be applied before interpolation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(transparent)]
pub struct OpaqueColor<CS> {
    /// The components, which may be manipulated directly.
    ///
    /// The interpretation of the components depends on the color space.
    pub components: [f32; 3],
    /// The color space.
    pub cs: PhantomData<CS>,
}

/// A color with an alpha channel.
///
/// A color in a color space known at compile time, with an alpha channel.
///
/// The color channels are straight, i.e., they are not premultiplied by
/// the alpha channel. See [`PremulColor`] for a color type with color
/// channels premultiplied by the alpha channel.
///
/// See [`OpaqueColor`] for a discussion of arithmetic traits and interpolation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(transparent)]
pub struct AlphaColor<CS> {
    /// The components, which may be manipulated directly.
    ///
    /// The interpretation of the first three components depends on the color
    /// space. The fourth component is separate alpha.
    pub components: [f32; 4],
    /// The color space.
    pub cs: PhantomData<CS>,
}

/// A color with premultiplied alpha.
///
/// A color in a color space known at compile time, with color channels
/// premultiplied by the alpha channel.
///
/// Following the convention of CSS Color 4, in cylindrical color spaces
/// the hue channel is not premultiplied. If it were, interpolation would
/// give undesirable results.
///
/// See [`AlphaColor`] for a color type without alpha premultiplication.
///
/// See [`OpaqueColor`] for a discussion of arithmetic traits and interpolation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(transparent)]
pub struct PremulColor<CS> {
    /// The components, which may be manipulated directly.
    ///
    /// The interpretation of the first three components depends on the color
    /// space, and are premultiplied with the alpha value. The fourth component
    /// is alpha.
    ///
    /// Note that in cylindrical color spaces, the hue component is not
    /// premultiplied, as specified in the CSS Color 4 spec. The methods on
    /// this type take care of that for you, but if you're manipulating the
    /// components yourself, be aware.
    pub components: [f32; 4],
    /// The color space.
    pub cs: PhantomData<CS>,
}

/// The hue direction for interpolation.
///
/// This type corresponds to [`hue-interpolation-method`] in the CSS Color
/// 4 spec.
///
/// [`hue-interpolation-method`]: https://developer.mozilla.org/en-US/docs/Web/CSS/hue-interpolation-method
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
#[repr(u8)]
pub enum HueDirection {
    /// Hue angles take the shorter of the two arcs between starting and ending values.
    #[default]
    Shorter = 0,
    /// Hue angles take the longer of the two arcs between starting and ending values.
    Longer = 1,
    /// Hue angles increase as they are interpolated.
    Increasing = 2,
    /// Hue angles decrease as they are interpolated.
    Decreasing = 3,
    // It's possible we'll add "raw"; color.js has it.
    // NOTICE: If a new value is added, be sure to modify `MAX_VALUE` in the bytemuck impl.
}

/// Defines how color channels should be handled when interpolating
/// between transparent colors.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(crate) enum InterpolationAlphaSpace {
    /// Colors are interpolated with their color channels premultiplied by the alpha
    /// channel. This is almost always what you want.
    ///
    /// Used when interpolating colors in the premultiplied alpha space, which allows
    /// for correct interpolation when colors are transparent. This matches behavior
    /// described in [CSS Color Module Level 4 § 12.3].
    ///
    /// Following the convention of CSS Color Module Level 4, in cylindrical color
    /// spaces the hue channel is not premultiplied. If it were, interpolation would
    /// give undesirable results. See also [`PremulColor`].
    ///
    /// [CSS Color Module Level 4 § 12.3]: https://drafts.csswg.org/css-color/#interpolation-alpha
    #[default]
    Premultiplied = 0,
    /// Colors are interpolated without premultiplying their color channels by the alpha channel.
    ///
    /// This causes color information to leak out of transparent colors. For example, when
    /// interpolating from a fully transparent red to a fully opaque blue in sRGB, this
    /// method will go through an intermediate purple.
    ///
    /// Used when interpolating colors in the unpremultiplied (straight) alpha space.
    /// This matches behavior of gradients in the HTML `canvas` element.
    /// See [The 2D rendering context § Fill and stroke styles].
    ///
    /// [The 2D rendering context § Fill and stroke styles]: https://html.spec.whatwg.org/multipage/#interpolation
    Unpremultiplied = 1,
}

impl InterpolationAlphaSpace {
    /// Returns `true` if the alpha mode is [`InterpolationAlphaSpace::Unpremultiplied`].
    pub(crate) const fn is_unpremultiplied(self) -> bool {
        matches!(self, Self::Unpremultiplied)
    }
}

/// Fixup hue based on specified hue direction.
///
/// Reference: §12.4 of CSS Color 4 spec
///
/// Note that this technique has been tweaked to only modify the second hue.
/// The rationale for this is to support multiple gradient stops, for example
/// in a spline. Apply the fixup to successive adjacent pairs.
///
/// In addition, hues outside [0, 360) are supported, with a resulting hue
/// difference always in [-360, 360].
fn fixup_hue(h1: f32, h2: &mut f32, direction: HueDirection) {
    let dh = (*h2 - h1) * (1. / 360.);
    match direction {
        HueDirection::Shorter => {
            // Round, resolving ties toward zero. This tricky formula
            // has been validated to yield the correct result for all
            // bit values of f32.
            *h2 -= 360. * ((dh.abs() - 0.25) - 0.25).ceil().copysign(dh);
        }
        HueDirection::Longer => {
            let t = 2.0 * dh.abs().ceil() - (dh.abs() + 1.5).floor();
            *h2 += 360.0 * (t.copysign(0.0 - dh));
        }
        HueDirection::Increasing => *h2 -= 360.0 * dh.floor(),
        HueDirection::Decreasing => *h2 -= 360.0 * dh.ceil(),
    }
}

pub(crate) fn fixup_hues_for_interpolate(
    a: [f32; 3],
    b: &mut [f32; 3],
    layout: ColorSpaceLayout,
    direction: HueDirection,
) {
    if let Some(ix) = layout.hue_channel() {
        fixup_hue(a[ix], &mut b[ix], direction);
    }
}

impl<CS: ColorSpace> OpaqueColor<CS> {
    /// A black color.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const BLACK: Self = Self::new([0., 0., 0.]);

    /// A white color.
    ///
    /// This value is specific to the color space.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const WHITE: Self = Self::new(CS::WHITE_COMPONENTS);

    /// Create a new color from the given components.
    pub const fn new(components: [f32; 3]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    /// Convert a color into a different color space.
    #[must_use]
    pub fn convert<TargetCS: ColorSpace>(self) -> OpaqueColor<TargetCS> {
        OpaqueColor::new(CS::convert::<TargetCS>(self.components))
    }

    /// Add an alpha channel.
    ///
    /// This function is the inverse of [`AlphaColor::split`].
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> AlphaColor<CS> {
        AlphaColor::new(add_alpha(self.components, alpha))
    }

    /// Difference between two colors by Euclidean metric.
    #[must_use]
    pub fn difference(self, other: Self) -> f32 {
        let d = (self - other).components;
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }

    /// Linearly interpolate colors, without hue fixup.
    ///
    /// This method produces meaningful results in rectangular color spaces,
    /// or if hue fixup has been applied.
    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self + t * (other - self)
    }

    /// Apply hue fixup for interpolation.
    ///
    /// Adjust the hue angle of `other` so that linear interpolation results in
    /// the expected hue direction.
    pub fn fixup_hues(self, other: &mut Self, direction: HueDirection) {
        fixup_hues_for_interpolate(
            self.components,
            &mut other.components,
            CS::LAYOUT,
            direction,
        );
    }

    /// Linearly interpolate colors, with hue fixup if needed.
    #[must_use]
    pub fn lerp(self, mut other: Self, t: f32, direction: HueDirection) -> Self {
        self.fixup_hues(&mut other, direction);
        self.lerp_rect(other, t)
    }

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        Self::new(CS::scale_chroma(self.components, scale))
    }

    /// Compute the relative luminance of the color.
    ///
    /// This can be useful for choosing contrasting colors, and follows the
    /// [WCAG 2.1 spec].
    ///
    /// [WCAG 2.1 spec]: https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
    #[must_use]
    pub fn relative_luminance(self) -> f32 {
        let [r, g, b] = CS::to_linear_srgb(self.components);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Map components.
    #[must_use]
    pub fn map(self, f: impl Fn(f32, f32, f32) -> [f32; 3]) -> Self {
        let [x, y, z] = self.components;
        Self::new(f(x, y, z))
    }

    /// Map components in a given color space.
    #[must_use]
    pub fn map_in<TargetCS: ColorSpace>(self, f: impl Fn(f32, f32, f32) -> [f32; 3]) -> Self {
        self.convert::<TargetCS>().map(f).convert()
    }

    /// Map the lightness of the color.
    ///
    /// In a color space that naturally has a lightness component, map that value.
    /// Otherwise, do the mapping in [Oklab]. The lightness range is normalized so
    /// that 1.0 is white. That is the normal range for [Oklab] but differs from the
    /// range in [Lab], [Lch], and [Hsl].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color::{Lab, OpaqueColor};
    ///
    /// let color = OpaqueColor::<Lab>::new([40., 4., -17.]);
    /// let lighter = color.map_lightness(|l| l + 0.2);
    /// let expected = OpaqueColor::<Lab>::new([60., 4., -17.]);
    ///
    /// assert!(lighter.difference(expected) < 1e-4);
    /// ```
    ///
    /// [Lab]: crate::Lab
    /// [Lch]: crate::Lch
    /// [Hsl]: crate::Hsl
    #[must_use]
    pub fn map_lightness(self, f: impl Fn(f32) -> f32) -> Self {
        match CS::TAG {
            Some(ColorSpaceTag::Lab) | Some(ColorSpaceTag::Lch) => {
                self.map(|l, c1, c2| [100.0 * f(l * 0.01), c1, c2])
            }
            Some(ColorSpaceTag::Oklab) | Some(ColorSpaceTag::Oklch) => {
                self.map(|l, c1, c2| [f(l), c1, c2])
            }
            Some(ColorSpaceTag::Hsl) => self.map(|h, s, l| [h, s, 100.0 * f(l * 0.01)]),
            _ => self.map_in::<Oklab>(|l, a, b| [f(l), a, b]),
        }
    }

    /// Map the hue of the color.
    ///
    /// In a color space that naturally has a hue component, map that value.
    /// Otherwise, do the mapping in [Oklch]. The hue is in degrees.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color::{Oklab, OpaqueColor};
    ///
    /// let color = OpaqueColor::<Oklab>::new([0.5, 0.2, -0.1]);
    /// let complementary = color.map_hue(|h| (h + 180.) % 360.);
    /// let expected = OpaqueColor::<Oklab>::new([0.5, -0.2, 0.1]);
    ///
    /// assert!(complementary.difference(expected) < 1e-4);
    /// ```
    #[must_use]
    pub fn map_hue(self, f: impl Fn(f32) -> f32) -> Self {
        match CS::LAYOUT {
            ColorSpaceLayout::HueFirst => self.map(|h, c1, c2| [f(h), c1, c2]),
            ColorSpaceLayout::HueThird => self.map(|c0, c1, h| [c0, c1, f(h)]),
            _ => self.map_in::<Oklch>(|l, c, h| [l, c, f(h)]),
        }
    }

    /// Convert the color to [sRGB][Srgb] if not already in sRGB, and pack into 8 bit per component
    /// integer encoding.
    ///
    /// The RGB components are mapped from the floating point range of `0.0-1.0` to the integer
    /// range of `0-255`. Component values outside of this range are saturated to 0 or 255. The
    /// alpha component is set to 255.
    ///
    /// # Implementation note
    ///
    /// This performs almost-correct rounding, see the note on [`AlphaColor::to_rgba8`].
    #[must_use]
    pub fn to_rgba8(self) -> Rgba8 {
        self.with_alpha(1.0).to_rgba8()
    }
}

pub(crate) const fn split_alpha([x, y, z, a]: [f32; 4]) -> ([f32; 3], f32) {
    ([x, y, z], a)
}

pub(crate) const fn add_alpha([x, y, z]: [f32; 3], a: f32) -> [f32; 4] {
    [x, y, z, a]
}

impl<CS: ColorSpace> AlphaColor<CS> {
    /// A black color.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const BLACK: Self = Self::new([0., 0., 0., 1.]);

    /// A transparent color.
    ///
    /// This is a black color with full alpha.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const TRANSPARENT: Self = Self::new([0., 0., 0., 0.]);

    /// A white color.
    ///
    /// This value is specific to the color space.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const WHITE: Self = Self::new(add_alpha(CS::WHITE_COMPONENTS, 1.));

    /// Create a new color from the given components.
    pub const fn new(components: [f32; 4]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    /// Split into opaque and alpha components.
    ///
    /// This function is the inverse of [`OpaqueColor::with_alpha`].
    #[must_use]
    pub const fn split(self) -> (OpaqueColor<CS>, f32) {
        let (opaque, alpha) = split_alpha(self.components);
        (OpaqueColor::new(opaque), alpha)
    }

    /// Set the alpha channel.
    ///
    /// This replaces the existing alpha channel. To scale or
    /// or otherwise modify the existing alpha channel, use
    /// [`AlphaColor::multiply_alpha`] or [`AlphaColor::map`].
    ///
    /// ```
    /// let c = color::palette::css::GOLDENROD.with_alpha(0.5);
    /// assert_eq!(0.5, c.split().1);
    /// ```
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        let (opaque, _alpha) = split_alpha(self.components);
        Self::new(add_alpha(opaque, alpha))
    }

    /// Split out the opaque components, discarding the alpha.
    ///
    /// This is a shorthand for calling [`split`](Self::split).
    #[must_use]
    pub const fn discard_alpha(self) -> OpaqueColor<CS> {
        self.split().0
    }

    /// Convert a color into a different color space.
    #[must_use]
    pub fn convert<TargetCs: ColorSpace>(self) -> AlphaColor<TargetCs> {
        let (opaque, alpha) = split_alpha(self.components);
        let components = CS::convert::<TargetCs>(opaque);
        AlphaColor::new(add_alpha(components, alpha))
    }

    /// Convert a color to the corresponding premultiplied form.
    #[must_use]
    pub const fn premultiply(self) -> PremulColor<CS> {
        let (opaque, alpha) = split_alpha(self.components);
        PremulColor::new(add_alpha(CS::LAYOUT.scale(opaque, alpha), alpha))
    }

    /// Difference between two colors by Euclidean metric.
    #[must_use]
    pub(crate) fn difference(self, other: Self) -> f32 {
        let d = (self - other).components;
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] + d[3] * d[3]).sqrt()
    }

    /// Linearly interpolate colors, without hue fixup.
    ///
    /// This method produces meaningful results in rectangular color spaces,
    /// or if hue fixup has been applied.
    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self.premultiply()
            .lerp_rect(other.premultiply(), t)
            .un_premultiply()
    }

    /// Linearly interpolate colors, with hue fixup if needed.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32, direction: HueDirection) -> Self {
        self.premultiply()
            .lerp(other.premultiply(), t, direction)
            .un_premultiply()
    }

    /// Multiply alpha by the given factor.
    #[must_use]
    pub const fn multiply_alpha(self, rhs: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        Self::new(add_alpha(opaque, alpha * rhs))
    }

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        Self::new(add_alpha(CS::scale_chroma(opaque, scale), alpha))
    }

    /// Map components.
    #[must_use]
    pub fn map(self, f: impl Fn(f32, f32, f32, f32) -> [f32; 4]) -> Self {
        let [x, y, z, a] = self.components;
        Self::new(f(x, y, z, a))
    }

    /// Map components in a given color space.
    #[must_use]
    pub fn map_in<TargetCS: ColorSpace>(self, f: impl Fn(f32, f32, f32, f32) -> [f32; 4]) -> Self {
        self.convert::<TargetCS>().map(f).convert()
    }

    /// Map the lightness of the color.
    ///
    /// In a color space that naturally has a lightness component, map that value.
    /// Otherwise, do the mapping in [Oklab]. The lightness range is normalized so
    /// that 1.0 is white. That is the normal range for [Oklab] but differs from the
    /// range in [Lab], [Lch], and [Hsl].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color::{AlphaColor, Lab};
    ///
    /// let color = AlphaColor::<Lab>::new([40., 4., -17., 1.]);
    /// let lighter = color.map_lightness(|l| l + 0.2);
    /// let expected = AlphaColor::<Lab>::new([60., 4., -17., 1.]);
    ///
    /// assert!(lighter.premultiply().difference(expected.premultiply()) < 1e-4);
    /// ```
    ///
    /// [Lab]: crate::Lab
    /// [Lch]: crate::Lch
    /// [Hsl]: crate::Hsl
    #[must_use]
    pub fn map_lightness(self, f: impl Fn(f32) -> f32) -> Self {
        match CS::TAG {
            Some(ColorSpaceTag::Lab) | Some(ColorSpaceTag::Lch) => {
                self.map(|l, c1, c2, a| [100.0 * f(l * 0.01), c1, c2, a])
            }
            Some(ColorSpaceTag::Oklab) | Some(ColorSpaceTag::Oklch) => {
                self.map(|l, c1, c2, a| [f(l), c1, c2, a])
            }
            Some(ColorSpaceTag::Hsl) => self.map(|h, s, l, a| [h, s, 100.0 * f(l * 0.01), a]),
            _ => self.map_in::<Oklab>(|l, a, b, alpha| [f(l), a, b, alpha]),
        }
    }

    /// Map the hue of the color.
    ///
    /// In a color space that naturally has a hue component, map that value.
    /// Otherwise, do the mapping in [Oklch]. The hue is in degrees.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color::{AlphaColor, Oklab};
    ///
    /// let color = AlphaColor::<Oklab>::new([0.5, 0.2, -0.1, 1.]);
    /// let complementary = color.map_hue(|h| (h + 180.) % 360.);
    /// let expected = AlphaColor::<Oklab>::new([0.5, -0.2, 0.1, 1.]);
    ///
    /// assert!(complementary.premultiply().difference(expected.premultiply()) < 1e-4);
    /// ```
    #[must_use]
    pub fn map_hue(self, f: impl Fn(f32) -> f32) -> Self {
        match CS::LAYOUT {
            ColorSpaceLayout::HueFirst => self.map(|h, c1, c2, a| [f(h), c1, c2, a]),
            ColorSpaceLayout::HueThird => self.map(|c0, c1, h, a| [c0, c1, f(h), a]),
            _ => self.map_in::<Oklch>(|l, c, h, alpha| [l, c, f(h), alpha]),
        }
    }

    /// Convert the color to [sRGB][Srgb] if not already in sRGB, and pack into 8 bit per component
    /// integer encoding.
    ///
    /// The RGBA components are mapped from the floating point range of `0.0-1.0` to the integer
    /// range of `0-255`. Component values outside of this range are saturated to 0 or 255.
    ///
    /// # Implementation note
    ///
    /// This performs almost-correct rounding to be fast on both x86 and AArch64 hardware. Within the
    /// saturated output range of this method, `0-255`, there is a single color component value
    /// where results differ: `0.0019607842`. This method maps that component to integer value `1`;
    /// it would more precisely be mapped to `0`.
    #[must_use]
    pub fn to_rgba8(self) -> Rgba8 {
        let [r, g, b, a] = self
            .convert::<Srgb>()
            .components
            .map(|x| fast_round_to_u8(x * 255.));
        Rgba8 { r, g, b, a }
    }
}

impl<CS: ColorSpace> PremulColor<CS> {
    /// A black color.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const BLACK: Self = Self::new([0., 0., 0., 1.]);

    /// A transparent color.
    ///
    /// This is a black color with full alpha.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const TRANSPARENT: Self = Self::new([0., 0., 0., 0.]);

    /// A white color.
    ///
    /// This value is specific to the color space.
    ///
    /// More comprehensive pre-defined colors are available
    /// in the [`color::palette`](crate::palette) module.
    pub const WHITE: Self = Self::new(add_alpha(CS::WHITE_COMPONENTS, 1.));

    /// Create a new color from the given components.
    pub const fn new(components: [f32; 4]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    /// Split out the opaque components, discarding the alpha.
    ///
    /// This is a shorthand for un-premultiplying the alpha and
    /// calling [`AlphaColor::discard_alpha`].
    ///
    /// The result of calling this on a fully transparent color
    /// will be the color black.
    #[must_use]
    pub const fn discard_alpha(self) -> OpaqueColor<CS> {
        self.un_premultiply().discard_alpha()
    }

    /// Convert a color into a different color space.
    #[must_use]
    pub fn convert<TargetCS: ColorSpace>(self) -> PremulColor<TargetCS> {
        if TypeId::of::<CS>() == TypeId::of::<TargetCS>() {
            PremulColor::new(self.components)
        } else if TargetCS::IS_LINEAR && CS::IS_LINEAR {
            let (multiplied, alpha) = split_alpha(self.components);
            let components = CS::convert::<TargetCS>(multiplied);
            PremulColor::new(add_alpha(components, alpha))
        } else {
            self.un_premultiply().convert().premultiply()
        }
    }

    /// Convert a color to the corresponding separate alpha form.
    #[must_use]
    pub const fn un_premultiply(self) -> AlphaColor<CS> {
        let (multiplied, alpha) = split_alpha(self.components);
        let scale = if alpha == 0.0 { 1.0 } else { 1.0 / alpha };
        AlphaColor::new(add_alpha(CS::LAYOUT.scale(multiplied, scale), alpha))
    }

    /// Interpolate colors.
    ///
    /// Note: this function doesn't fix up hue in cylindrical spaces. It is
    /// still useful if the hue angles are compatible, particularly if the
    /// fixup has been applied.
    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self + t * (other - self)
    }

    /// Apply hue fixup for interpolation.
    ///
    /// Adjust the hue angle of `other` so that linear interpolation results in
    /// the expected hue direction.
    pub fn fixup_hues(self, other: &mut Self, direction: HueDirection) {
        if let Some(ix) = CS::LAYOUT.hue_channel() {
            fixup_hue(self.components[ix], &mut other.components[ix], direction);
        }
    }

    /// Linearly interpolate colors, with hue fixup if needed.
    #[must_use]
    pub fn lerp(self, mut other: Self, t: f32, direction: HueDirection) -> Self {
        self.fixup_hues(&mut other, direction);
        self.lerp_rect(other, t)
    }

    /// Multiply alpha by the given factor.
    #[must_use]
    pub const fn multiply_alpha(self, rhs: f32) -> Self {
        let (multiplied, alpha) = split_alpha(self.components);
        Self::new(add_alpha(CS::LAYOUT.scale(multiplied, rhs), alpha * rhs))
    }

    /// Difference between two colors by Euclidean metric.
    #[must_use]
    pub fn difference(self, other: Self) -> f32 {
        let d = (self - other).components;
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] + d[3] * d[3]).sqrt()
    }

    /// Convert the color to [sRGB][Srgb] if not already in sRGB, and pack into 8 bit per component
    /// integer encoding.
    ///
    /// The RGBA components are mapped from the floating point range of `0.0-1.0` to the integer
    /// range of `0-255`. Component values outside of this range are saturated to 0 or 255.
    ///
    /// # Implementation note
    ///
    /// This performs almost-correct rounding, see the note on [`AlphaColor::to_rgba8`].
    #[must_use]
    pub fn to_rgba8(self) -> PremulRgba8 {
        let [r, g, b, a] = self
            .convert::<Srgb>()
            .components
            .map(|x| fast_round_to_u8(x * 255.));
        PremulRgba8 { r, g, b, a }
    }
}

/// Fast rounding of `f32` to integer `u8`, rounding ties up.
///
/// Targeting x86, `f32::round` calls out to libc `roundf`. Even if that call were inlined, it is
/// branchy, which would make it relatively slow. The following is faster, and on the range `0-255`
/// almost correct*. AArch64 has dedicated rounding instructions so does not need this
/// optimization, but the following is still fast.
///
/// * The only input where the output differs from `a.round() as u8` is `0.49999997`.
#[inline(always)]
#[expect(clippy::cast_possible_truncation, reason = "deliberate quantization")]
fn fast_round_to_u8(a: f32) -> u8 {
    // This does not need clamping as the behavior of a `f32` to `u8` cast in Rust is to saturate.
    (a + 0.5) as u8
}

// Lossless conversion traits.

impl<CS: ColorSpace> From<OpaqueColor<CS>> for AlphaColor<CS> {
    fn from(value: OpaqueColor<CS>) -> Self {
        value.with_alpha(1.0)
    }
}

impl<CS: ColorSpace> From<OpaqueColor<CS>> for PremulColor<CS> {
    fn from(value: OpaqueColor<CS>) -> Self {
        Self::new(add_alpha(value.components, 1.0))
    }
}

// Partial equality - Hand derive to avoid needing ColorSpace to be PartialEq

impl<CS: ColorSpace> PartialEq for AlphaColor<CS> {
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

impl<CS: ColorSpace> PartialEq for OpaqueColor<CS> {
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

impl<CS: ColorSpace> PartialEq for PremulColor<CS> {
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<f32> for OpaqueColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<OpaqueColor<CS>> for f32 {
    type Output = OpaqueColor<CS>;

    fn mul(self, rhs: OpaqueColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for OpaqueColor<CS> {
    type Output = Self;

    // https://github.com/rust-lang/rust-clippy/issues/13652 has been filed
    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for OpaqueColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for OpaqueColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2]])
    }
}

impl<CS> BitEq for OpaqueColor<CS> {
    fn bit_eq(&self, other: &Self) -> bool {
        self.components.bit_eq(&other.components)
    }
}

impl<CS> BitHash for OpaqueColor<CS> {
    fn bit_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.components.bit_hash(state);
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<f32> for AlphaColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<AlphaColor<CS>> for f32 {
    type Output = AlphaColor<CS>;

    fn mul(self, rhs: AlphaColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for AlphaColor<CS> {
    type Output = Self;

    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for AlphaColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2], x[3] + y[3]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for AlphaColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2], x[3] - y[3]])
    }
}

impl<CS> BitEq for AlphaColor<CS> {
    fn bit_eq(&self, other: &Self) -> bool {
        self.components.bit_eq(&other.components)
    }
}

impl<CS> BitHash for AlphaColor<CS> {
    fn bit_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.components.bit_hash(state);
    }
}

/// Multiply components by a scalar.
///
/// For rectangular color spaces, this is equivalent to multiplying
/// alpha, but for cylindrical color spaces, [`PremulColor::multiply_alpha`]
/// is the preferred method.
impl<CS: ColorSpace> core::ops::Mul<f32> for PremulColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<PremulColor<CS>> for f32 {
    type Output = PremulColor<CS>;

    fn mul(self, rhs: PremulColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for PremulColor<CS> {
    type Output = Self;

    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for PremulColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2], x[3] + y[3]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for PremulColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2], x[3] - y[3]])
    }
}

impl<CS> BitEq for PremulColor<CS> {
    fn bit_eq(&self, other: &Self) -> bool {
        self.components.bit_eq(&other.components)
    }
}

impl<CS> BitHash for PremulColor<CS> {
    fn bit_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.components.bit_hash(state);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::{
        fast_round_to_u8, fixup_hue, AlphaColor, HueDirection, PremulColor, PremulRgba8, Rgba8,
        Srgb,
    };

    #[test]
    fn to_rgba8_saturation() {
        // This is just testing the Rust compiler behavior described in
        // <https://github.com/rust-lang/rust/issues/10184>.
        let (r, g, b, a) = (0, 0, 255, 255);

        let ac = AlphaColor::<Srgb>::new([-1.01, -0.5, 1.01, 2.0]);
        assert_eq!(ac.to_rgba8(), Rgba8 { r, g, b, a });

        let pc = PremulColor::<Srgb>::new([-1.01, -0.5, 1.01, 2.0]);
        assert_eq!(pc.to_rgba8(), PremulRgba8 { r, g, b, a });
    }

    #[test]
    fn hue_fixup() {
        // Verify that the hue arc matches the spec for all hues specified
        // within [0,360).
        for h1 in [0.0, 10.0, 180.0, 190.0, 350.0] {
            for h2 in [0.0, 10.0, 180.0, 190.0, 350.0] {
                let dh = h2 - h1;
                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Shorter);
                    let (mut spec_h1, mut spec_h2) = (h1, h2);
                    if dh > 180.0 {
                        spec_h1 += 360.0;
                    } else if dh < -180.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Longer);
                    let (mut spec_h1, mut spec_h2) = (h1, h2);
                    if 0.0 < dh && dh < 180.0 {
                        spec_h1 += 360.0;
                    } else if -180.0 < dh && dh <= 0.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Increasing);
                    let (spec_h1, mut spec_h2) = (h1, h2);
                    if dh < 0.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Decreasing);
                    let (mut spec_h1, spec_h2) = (h1, h2);
                    if dh > 0.0 {
                        spec_h1 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }
            }
        }
    }

    /// Test the claim in [`super::fast_round_to_u8`] that the only rounding failure in the range
    /// of interest occurs for `0.49999997`.
    #[test]
    fn fast_round() {
        #[expect(clippy::cast_possible_truncation, reason = "deliberate quantization")]
        fn real_round_to_u8(v: f32) -> u8 {
            v.round() as u8
        }

        // Check the rounding behavior at integer and half integer values within (and near) the
        // range 0-255, as well as one ULP up and down from those values.
        let mut failures = alloc::vec![];
        let mut v = -1_f32;

        while v <= 256. {
            // Note we don't get accumulation of rounding errors by incrementing with 0.5: integers
            // and half integers are exactly representable in this range.
            assert!(v.abs().fract() == 0. || v.abs().fract() == 0.5, "{v}");

            let mut validate_rounding = |val: f32| {
                if real_round_to_u8(val) != fast_round_to_u8(val) {
                    failures.push(val);
                }
            };

            validate_rounding(v.next_down().next_down());
            validate_rounding(v.next_down());
            validate_rounding(v);
            validate_rounding(v.next_up());
            validate_rounding(v.next_up().next_up());

            v += 0.5;
        }

        assert_eq!(&failures, &[0.49999997]);
    }

    /// A more thorough test than the one above: the one above only tests values that are likely to
    /// fail. This test runs through all floats in and near the range of interest (approximately
    /// 200 million floats), so can be somewhat slow (seconds rather than milliseconds). To run
    /// this test, use the `--ignored` flag.
    #[test]
    #[ignore = "Takes too long to execute."]
    fn fast_round_full() {
        #[expect(clippy::cast_possible_truncation, reason = "deliberate quantization")]
        fn real_round_to_u8(v: f32) -> u8 {
            v.round() as u8
        }

        // Check the rounding behavior of all floating point values within (and near) the range
        // 0-255.
        let mut failures = alloc::vec![];
        let mut v = -1_f32;

        while v <= 256. {
            if real_round_to_u8(v) != fast_round_to_u8(v) {
                failures.push(v);
            }
            v = v.next_up();
        }

        assert_eq!(&failures, &[0.49999997]);
    }
}
