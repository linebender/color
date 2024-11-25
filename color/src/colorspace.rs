// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{any::TypeId, f32};

use crate::{matmul, tag::ColorSpaceTag};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

/// The main trait for color spaces.
///
/// This can be implemented by clients for conversions in and out of
/// new color spaces. It is expected to be a zero-sized type.
///
/// The [linear sRGB](`LinearSrgb`) color space is central, and other
/// color spaces are defined as conversions in and out of that. A color
/// space does not explicitly define a gamut, so generally conversions
/// will succeed and round-trip, subject to numerical precision.
///
/// White point is not explicitly represented. For color spaces with a
/// white point other than D65 (the native white point for sRGB), use
/// a linear Bradford chromatic adaptation, following CSS Color 4.
///
/// See the [XYZ-D65 color space](`XyzD65`) documentation for some
/// background information on color spaces.
pub trait ColorSpace: Clone + Copy + 'static {
    /// Whether the color space is linear.
    ///
    /// Calculations in linear color spaces can sometimes be simplified,
    /// for example it is not necessary to undo premultiplication when
    /// converting.
    const IS_LINEAR: bool = false;

    /// The layout of the color space.
    ///
    /// The layout primarily identifies the hue channel for cylindrical
    /// color spaces, which is important because hue is not premultiplied.
    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::Rectangular;

    /// The tag corresponding to this color space, if a matching tag exists.
    const TAG: Option<ColorSpaceTag> = None;

    /// The component values for the color white within this color space.
    const WHITE_COMPONENTS: [f32; 3];

    /// Convert an opaque color to linear sRGB.
    ///
    /// Values are likely to exceed [0, 1] for wide-gamut and HDR colors.
    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3];

    /// Convert an opaque color from linear sRGB.
    ///
    /// In general, this method should not do any gamut clipping.
    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3];

    /// Scale the chroma by the given amount.
    ///
    /// In color spaces with a natural representation of chroma, scale
    /// directly. In other color spaces, equivalent results as scaling
    /// chroma in Oklab.
    fn scale_chroma(src: [f32; 3], scale: f32) -> [f32; 3] {
        let rgb = Self::to_linear_srgb(src);
        let scaled = LinearSrgb::scale_chroma(rgb, scale);
        Self::from_linear_srgb(scaled)
    }

    /// Convert to a different color space.
    ///
    /// The default implementation is a no-op if the color spaces
    /// are the same, otherwise converts from the source to linear
    /// sRGB, then from that to the target. Implementations are
    /// encouraged to specialize further (using the [`TypeId`] of
    /// the color spaces), effectively finding a shortest path in
    /// the conversion graph.
    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    /// Clip the color's components to fit within the natural gamut of the color space.
    ///
    /// There are many possible ways to map colors outside of a color space's gamut to colors
    /// inside the gamut. Some methods are perceptually better than others (for example, preserving
    /// the mapped color's hue is usually preferred over preserving saturation). This method will
    /// generally do the mathematically simplest thing, namely clamping the individual color
    /// components' values to the color space's natural limits of those components, bringing
    /// out-of-gamut colors just onto the gamut boundary. The resultant color may be perceptually
    /// quite distinct from the original color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color::{ColorSpace, Srgb, XyzD65};
    ///
    /// assert_eq!(Srgb::clip([0.4, -0.2, 1.2]), [0.4, 0., 1.]);
    /// assert_eq!(XyzD65::clip([0.4, -0.2, 1.2]), [0.4, -0.2, 1.2]);
    /// ```
    fn clip(src: [f32; 3]) -> [f32; 3];
}

/// The layout of a color space, particularly the hue component.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ColorSpaceLayout {
    /// Rectangular, no hue component.
    Rectangular,
    /// Cylindrical, hue is first component.
    HueFirst,
    /// Cylindrical, hue is third component.
    HueThird,
}

impl ColorSpaceLayout {
    /// Multiply all components except for hue by scale.
    ///
    /// This function is used for both premultiplying and un-premultiplying. See
    /// §12.3 of Color 4 spec for context.
    pub(crate) const fn scale(self, components: [f32; 3], scale: f32) -> [f32; 3] {
        match self {
            Self::Rectangular => [
                components[0] * scale,
                components[1] * scale,
                components[2] * scale,
            ],
            Self::HueFirst => [components[0], components[1] * scale, components[2] * scale],
            Self::HueThird => [components[0] * scale, components[1] * scale, components[2]],
        }
    }

    pub(crate) const fn hue_channel(self) -> Option<usize> {
        match self {
            Self::Rectangular => None,
            Self::HueFirst => Some(0),
            Self::HueThird => Some(2),
        }
    }
}

/// 🌌 The linear-light RGB color space with [sRGB](`Srgb`) primaries.
///
/// This color space is identical to sRGB, having the same components and natural gamut, except
/// that the transfer function is linear.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the channels are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.3][css-sec].
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-sRGB-linear
#[derive(Clone, Copy, Debug)]
pub struct LinearSrgb;

impl ColorSpace for LinearSrgb {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::LinearSrgb);

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src
    }

    fn scale_chroma(src: [f32; 3], scale: f32) -> [f32; 3] {
        let lms = matmul(&OKLAB_SRGB_TO_LMS, src).map(f32::cbrt);
        let l = OKLAB_LMS_TO_LAB[0];
        let lightness = l[0] * lms[0] + l[1] * lms[1] + l[2] * lms[2];
        let lms_scaled = [
            lightness + scale * (lms[0] - lightness),
            lightness + scale * (lms[1] - lightness),
            lightness + scale * (lms[2] - lightness),
        ];
        matmul(&OKLAB_LMS_TO_SRGB, lms_scaled.map(|x| x * x * x))
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The standard RGB color space.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the components are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.2][css-sec]. It is
/// defined in IEC 61966-2-1.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-sRGB
#[derive(Clone, Copy, Debug)]
pub struct Srgb;

fn srgb_to_lin(x: f32) -> f32 {
    if x.abs() <= 0.04045 {
        x * (1.0 / 12.92)
    } else {
        ((x.abs() + 0.055) * (1.0 / 1.055)).powf(2.4).copysign(x)
    }
}

fn lin_to_srgb(x: f32) -> f32 {
    if x.abs() <= 0.0031308 {
        x * 12.92
    } else {
        (1.055 * x.abs().powf(1.0 / 2.4) - 0.055).copysign(x)
    }
}

impl ColorSpace for Srgb {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Srgb);

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(srgb_to_lin)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(lin_to_srgb)
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Hsl>() {
            rgb_to_hsl(src, true)
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Hwb>() {
            rgb_to_hwb(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The Display P3 color space, often used for wide-gamut displays.
///
/// Display P3 is similar to [sRGB](`Srgb`) but has higher red and, especially, green
/// chromaticities, thereby extending its gamut over sRGB on those components.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the channels are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.4][css-sec] and is
/// [characterized by the ICC][icc]. Display P3 is a variant of the DCI-P3 color space
/// described in [SMPTE EG 432-1:2010][smpte].
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-display-p3
/// [icc]: https://www.color.org/chardata/rgb/DisplayP3.xalter
/// [smpte]: https://pub.smpte.org/doc/eg432-1/20101110-pub/eg0432-1-2010.pdf
#[derive(Clone, Copy, Debug)]
pub struct DisplayP3;

impl ColorSpace for DisplayP3 {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::DisplayP3);

    const WHITE_COMPONENTS: [f32; 3] = [0.99999994, 0.99999994, 0.99999994];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        const LINEAR_DISPLAYP3_TO_SRGB: [[f32; 3]; 3] = [
            [1.224_940_2, -0.224_940_18, 0.0],
            [-0.042_056_955, 1.042_056_9, 0.0],
            [-0.019_637_555, -0.078_636_04, 1.098_273_6],
        ];
        matmul(&LINEAR_DISPLAYP3_TO_SRGB, src.map(srgb_to_lin))
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        const LINEAR_SRGB_TO_DISPLAYP3: [[f32; 3]; 3] = [
            [0.822_461_96, 0.177_538_04, 0.0],
            [0.033_194_2, 0.966_805_8, 0.0],
            [0.017_082_632, 0.072_397_44, 0.910_519_96],
        ];
        matmul(&LINEAR_SRGB_TO_DISPLAYP3, src).map(lin_to_srgb)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The Adobe RGB (1998) color space.
///
/// Adobe RGB is similar to [sRGB](`Srgb`) but has higher green chromaticity, thereby extending its
/// gamut over sRGB on that component. It was developed to encompass typical color print gamuts.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the channels are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.5][css-sec] and is
/// [characterized by the ICC][icc]. Adobe RGB is described [here][adobe] by Adobe.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-a98-rgb
/// [icc]: https://www.color.org/chardata/rgb/adobergb.xalter
/// [adobe]: https://www.adobe.com/digitalimag/adobergb.html
#[derive(Clone, Copy, Debug)]
pub struct A98Rgb;

impl ColorSpace for A98Rgb {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::A98Rgb);

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

    fn to_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * lin_A98_to_XYZ
        #[expect(
            clippy::cast_possible_truncation,
            reason = "exact rational, truncate at compile-time"
        )]
        const LINEAR_A98RGB_TO_SRGB: [[f32; 3]; 3] = [
            [
                (66_942_405. / 47_872_228.) as f32,
                (-19_070_177. / 47_872_228.) as f32,
                0.,
            ],
            [0., 1., 0.],
            [
                0.,
                (-11_512_411. / 268_173_353.) as f32,
                (279_685_764. / 268_173_353.) as f32,
            ],
        ];
        matmul(
            &LINEAR_A98RGB_TO_SRGB,
            [r, g, b].map(|x| x.abs().powf(563. / 256.).copysign(x)),
        )
    }

    fn from_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_A98RGB * lin_sRGB_to_XYZ
        #[expect(
            clippy::cast_possible_truncation,
            reason = "exact rational, truncate at compile-time"
        )]
        const LINEAR_SRGB_TO_A98RGB: [[f32; 3]; 3] = [
            [
                (47_872_228. / 66_942_405.) as f32,
                (19_070_177. / 66_942_405.) as f32,
                0.0,
            ],
            [0., 1., 0.],
            [
                0.,
                (11_512_411. / 279_685_764.) as f32,
                (268_173_353. / 279_685_764.) as f32,
            ],
        ];
        matmul(&LINEAR_SRGB_TO_A98RGB, [r, g, b]).map(|x| x.abs().powf(256. / 563.).copysign(x))
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The ProPhoto RGB color space.
///
/// ProPhoto RGB is similar to [sRGB](`Srgb`) but has higher red, green and blue chromaticities,
/// thereby extending its gamut over sRGB on all components. ProPhoto RGB has a reference white of
/// D50; see the [XYZ-D65 color space](`XyzD65`) documentation for some background information on
/// the meaning of "reference white."
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the channels are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.6][css-sec] and is
/// [characterized by the ICC][icc].
///
/// ProPhoto RGB is also known as ROMM RGB.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-prophoto-rgb
/// [icc]: https://www.color.org/chardata/rgb/rommrgb.xalter
#[derive(Clone, Copy, Debug)]
pub struct ProphotoRgb;

impl ColorSpace for ProphotoRgb {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::ProphotoRgb);

    const WHITE_COMPONENTS: [f32; 3] = [1., 0.99999994, 1.];

    fn to_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * D50_to_D65 * lin_prophoto_to_XYZ
        const LINEAR_PROPHOTORGB_TO_SRGB: [[f32; 3]; 3] = [
            [2.034_367_6, -0.727_634_5, -0.306_733_07],
            [-0.228_826_79, 1.231_753_3, -0.002_926_598],
            [-0.008_558_424, -0.153_268_2, 1.161_826_6],
        ];

        fn transfer(x: f32) -> f32 {
            if x.abs() <= 16. / 512. {
                x / 16.
            } else {
                x.abs().powf(1.8).copysign(x)
            }
        }

        matmul(&LINEAR_PROPHOTORGB_TO_SRGB, [r, g, b].map(transfer))
    }

    fn from_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_prophoto * D65_to_D50 * lin_sRGB_to_XYZ
        const LINEAR_SRGB_TO_PROPHOTORGB: [[f32; 3]; 3] = [
            [0.529_280_4, 0.330_153, 0.140_566_6],
            [0.098_366_22, 0.873_463_9, 0.028_169_824],
            [0.016_875_342, 0.117_659_41, 0.865_465_2],
        ];

        fn transfer(x: f32) -> f32 {
            if x.abs() <= 1. / 512. {
                x * 16.
            } else {
                x.abs().powf(1. / 1.8).copysign(x)
            }
        }

        matmul(&LINEAR_SRGB_TO_PROPHOTORGB, [r, g, b]).map(transfer)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The Rec. 2020 color space.
///
/// Rec. 2020 is similar to [sRGB](`Srgb`) but has higher red, green and blue chromaticities,
/// thereby extending its gamut over sRGB on all components.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the channels are `[0, 1]`.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.7][css-sec] and is
/// [characterized by the ICC][icc]. The color space is defined by the International
/// Telecommunication Union [here][itu].
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-rec2020
/// [icc]: https://www.color.org/chardata/rgb/BT2020.xalter
/// [itu]: https://www.itu.int/rec/R-REC-BT.2020/en
#[derive(Clone, Copy, Debug)]
pub struct Rec2020;

impl Rec2020 {
    // These are the parameters of the transfer function defined in the Rec. 2020 specification.
    // They are truncated here to f32 precision.
    const A: f32 = 1.099_296_8;
    const B: f32 = 0.018_053_97;
}

impl ColorSpace for Rec2020 {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Rec2020);

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

    fn to_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * lin_Rec2020_to_XYZ
        #[expect(
            clippy::cast_possible_truncation,
            reason = "exact rational, truncate at compile-time"
        )]
        const LINEAR_REC2020_TO_SRGB: [[f32; 3]; 3] = [
            [
                (2_785_571_537. / 1_677_558_947.) as f32,
                (-985_802_650. / 1_677_558_947.) as f32,
                (-122_209_940. / 1_677_558_947.) as f32,
            ],
            [
                (-4_638_020_506. / 37_238_079_773.) as f32,
                (42_187_016_744. / 37_238_079_773.) as f32,
                (-310_916_465. / 37_238_079_773.) as f32,
            ],
            [
                (-97_469_024. / 5_369_968_309.) as f32,
                (-3_780_738_464. / 37_589_778_163.) as f32,
                (42_052_799_795. / 37_589_778_163.) as f32,
            ],
        ];

        fn transfer(x: f32) -> f32 {
            if x.abs() < Rec2020::B * 4.5 {
                x * (1. / 4.5)
            } else {
                ((x.abs() + (Rec2020::A - 1.)) / Rec2020::A)
                    .powf(1. / 0.45)
                    .copysign(x)
            }
        }

        matmul(&LINEAR_REC2020_TO_SRGB, [r, g, b].map(transfer))
    }

    fn from_linear_srgb([r, g, b]: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_Rec2020 * lin_sRGB_to_XYZ
        #[expect(
            clippy::cast_possible_truncation,
            reason = "exact rational, truncate at compile-time"
        )]
        const LINEAR_SRGB_TO_REC2020: [[f32; 3]; 3] = [
            [
                (2_939_026_994. / 4_684_425_795.) as f32,
                (9_255_011_753. / 28_106_554_770.) as f32,
                (173_911_579. / 4_015_222_110.) as f32,
            ],
            [
                (76_515_593. / 1_107_360_270.) as f32,
                (6_109_575_001. / 6_644_161_620.) as f32,
                (75_493_061. / 6_644_161_620.) as f32,
            ],
            [
                (12_225_392. / 745_840_075.) as f32,
                (1_772_384_008. / 20_137_682_025.) as f32,
                (18_035_212_433. / 20_137_682_025.) as f32,
            ],
        ];

        fn transfer(x: f32) -> f32 {
            if x.abs() < Rec2020::B {
                x * 4.5
            } else {
                (Rec2020::A * x.abs().powf(0.45) - (Rec2020::A - 1.)).copysign(x)
            }
        }
        matmul(&LINEAR_SRGB_TO_REC2020, [r, g, b]).map(transfer)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// 🌌 The CIE XYZ color space with a 2° observer and a reference white of D50.
///
/// Its components are `[X, Y, Z]`. The components are unbounded, but are usually positive.
/// Reference white has a luminance `Y` of 1.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.8][css-sec]. It is
/// defined in CIE 015:2018. Following [CSS Color Module Level 4 § 11][css-chromatic-adaptation],
/// the conversion between D50 and D65 white points is done with the standard Bradford linear
/// chromatic adaptation transform.
///
/// See the [XYZ-D65 color space](`XyzD65`) documentation for some background information on color
/// spaces.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-xyz
/// [css-chromatic-adaptation]: https://www.w3.org/TR/css-color-4/#color-conversion
#[derive(Clone, Copy, Debug)]
pub struct XyzD50;

impl ColorSpace for XyzD50 {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::XyzD50);

    const WHITE_COMPONENTS: [f32; 3] = [0.9642956, 1., 0.8251046];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * D50_to_D65
        const XYZ_TO_LINEAR_SRGB: [[f32; 3]; 3] = [
            [3.134_136, -1.617_386, -0.490_662_22],
            [-0.978_795_47, 1.916_254_4, 0.033_442_874],
            [0.071_955_39, -0.228_976_76, 1.405_386_1],
        ];
        matmul(&XYZ_TO_LINEAR_SRGB, src)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // D65_to_D50 * lin_sRGB_to_XYZ
        const LINEAR_SRGB_TO_XYZ: [[f32; 3]; 3] = [
            [0.436_065_73, 0.385_151_5, 0.143_078_42],
            [0.222_493_17, 0.716_887, 0.060_619_81],
            [0.013_923_922, 0.097_081_326, 0.714_099_35],
        ];
        matmul(&LINEAR_SRGB_TO_XYZ, src)
    }

    fn clip([x, y, z]: [f32; 3]) -> [f32; 3] {
        [x, y, z]
    }
}

/// 🌌 The CIE XYZ color space with a 2° observer and a reference white of D65.
///
/// Its components are `[X, Y, Z]`. The components are unbounded, but are usually positive.
/// Reference white has a luminance `Y` of 1.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.8][css-sec]. It is
/// defined in CIE 015:2018. Following [CSS Color Module Level 4 § 11][css-chromatic-adaptation],
/// the conversion between D50 and D65 white points is done with the standard Bradford linear
/// chromatic adaptation transform.
///
/// # Human color vision and color spaces
///
/// Human color vision uses three types of photoreceptive cell in the eye that are sensitive to
/// light. These cells have their peak sensitivity at different wavelengths of light: roughly 570
/// nm, 535 nm and 430 nm, usually named Long, Medium and Short (LMS) respectively. The cells'
/// sensitivities to light taper off as the wavelength moves away from their peaks, but all three
/// cells overlap in wavelength sensitivity.
///
/// Visible light with a combination of wavelengths at specific intensities (the light's *spectral
/// density*), causes excitation of these three cell types in varying amounts. The human brain
/// interprets this as a specific color at a certain luminosity. Importantly, humans do not
/// directly perceive the light's wavelength: for example, monochromatic light with a wavelength of
/// 580 nm is perceived as "yellow," and light made up of two wavelengths at roughly 550nm
/// ("green") and 610 nm ("red") is also perceived as "yellow."
///
/// The CIE XYZ color space is an experimentally-obtained mapping of monochromatic light at a
/// specific wavelength to the response of human L, M and S photoreceptive cells (with some
/// additional mathematically desirable properties). Light of a specific spectral density maps onto
/// a specific coordinate in the XYZ color space. Light of a different spectral density that maps
/// onto the same XYZ coordinate is predicted by the color space to be perceived as the same
/// color and luminosity.
///
/// The XYZ color space is often used in the characterization of other color spaces.
///
/// ## White point
///
/// An important concept in color spaces is the *white point*. Whereas pure black is the absence of
/// illumination and has a natural representation in additive color spaces, white is more difficult
/// to define. CIE D65 defines white as the perceived color of diffuse standard noon daylight
/// perfectly reflected off a surface observed under some foveal angle; here 2°.
///
/// In many color spaces, their white point is the brightest illumination they can naturally
/// represent.
///
/// For further reading, the [Wikipedia article on the CIE XYZ color space][wikipedia-cie] provides
/// a good introduction to color theory as relevant to color spaces.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#predefined-xyz
/// [css-chromatic-adaptation]: https://www.w3.org/TR/css-color-4/#color-conversion
/// [wikipedia-cie]: https://en.wikipedia.org/wiki/CIE_1931_color_space
#[derive(Clone, Copy, Debug)]
pub struct XyzD65;

impl ColorSpace for XyzD65 {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::XyzD65);

    const WHITE_COMPONENTS: [f32; 3] = [0.9504559, 1., 1.0890577];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        const XYZ_TO_LINEAR_SRGB: [[f32; 3]; 3] = [
            [3.240_97, -1.537_383_2, -0.498_610_76],
            [-0.969_243_65, 1.875_967_5, 0.041_555_06],
            [0.055_630_08, -0.203_976_96, 1.056_971_5],
        ];
        matmul(&XYZ_TO_LINEAR_SRGB, src)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        const LINEAR_SRGB_TO_XYZ: [[f32; 3]; 3] = [
            [0.412_390_8, 0.357_584_33, 0.180_480_8],
            [0.212_639, 0.715_168_65, 0.072_192_32],
            [0.019_330_818, 0.119_194_78, 0.950_532_14],
        ];
        matmul(&LINEAR_SRGB_TO_XYZ, src)
    }

    fn clip([x, y, z]: [f32; 3]) -> [f32; 3] {
        [x, y, z]
    }
}

/// 🌌 The Oklab color space, intended to be a perceptually uniform color space.
///
/// Its components are `[L, a, b]` with
/// - `L` - the lightness with a natural bound between 0 and 1, where 0 represents pure black and 1
///    represents the lightness of white;
/// - `a` - how green/red the color is; and
/// - `b` - how blue/yellow the color is.
///
/// `a` and `b` are unbounded, but are usually between -0.5 and 0.5.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 9.2 ][css-sec]. It is
/// defined on [Björn Ottosson's blog][bjorn]. It is similar to the [CIELAB] color space but with
/// improved hue constancy.
///
/// Oklab has a cylindrical counterpart: [Oklch](`Oklch`).
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#ok-lab
/// [bjorn]: https://bottosson.github.io/posts/oklab/
/// [CIELAB]: Lab
#[derive(Clone, Copy, Debug)]
pub struct Oklab;

// Matrices taken from [Oklab] blog post, precision reduced to f32
//
// [Oklab]: https://bottosson.github.io/posts/oklab/
const OKLAB_LAB_TO_LMS: [[f32; 3]; 3] = [
    [1.0, 0.396_337_78, 0.215_803_76],
    [1.0, -0.105_561_346, -0.063_854_17],
    [1.0, -0.089_484_18, -1.291_485_5],
];

const OKLAB_LMS_TO_SRGB: [[f32; 3]; 3] = [
    [4.076_741_7, -3.307_711_6, 0.230_969_94],
    [-1.268_438, 2.609_757_4, -0.341_319_38],
    [-0.004_196_086_3, -0.703_418_6, 1.707_614_7],
];

const OKLAB_SRGB_TO_LMS: [[f32; 3]; 3] = [
    [0.412_221_46, 0.536_332_55, 0.051_445_995],
    [0.211_903_5, 0.680_699_5, 0.107_396_96],
    [0.088_302_46, 0.281_718_85, 0.629_978_7],
];

const OKLAB_LMS_TO_LAB: [[f32; 3]; 3] = [
    [0.210_454_26, 0.793_617_8, -0.004_072_047],
    [1.977_998_5, -2.428_592_2, 0.450_593_7],
    [0.025_904_037, 0.782_771_77, -0.808_675_77],
];

impl ColorSpace for Oklab {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Oklab);

    const WHITE_COMPONENTS: [f32; 3] = [1., 0., 0.];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let lms = matmul(&OKLAB_LAB_TO_LMS, src).map(|x| x * x * x);
        matmul(&OKLAB_LMS_TO_SRGB, lms)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let lms = matmul(&OKLAB_SRGB_TO_LMS, src).map(f32::cbrt);
        matmul(&OKLAB_LMS_TO_LAB, lms)
    }

    fn scale_chroma([l, a, b]: [f32; 3], scale: f32) -> [f32; 3] {
        [l, a * scale, b * scale]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Oklch>() {
            lab_to_lch(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, a, b]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 1.), a, b]
    }
}

/// Rectangular to cylindrical conversion.
fn lab_to_lch([l, a, b]: [f32; 3]) -> [f32; 3] {
    let mut h = b.atan2(a) * (180. / f32::consts::PI);
    if h < 0.0 {
        h += 360.0;
    }
    let c = b.hypot(a);
    [l, c, h]
}

/// Cylindrical to rectangular conversion.
fn lch_to_lab([l, c, h]: [f32; 3]) -> [f32; 3] {
    let (sin, cos) = (h * (f32::consts::PI / 180.)).sin_cos();
    let a = c * cos;
    let b = c * sin;
    [l, a, b]
}

/// 🌌 The cylindrical version of the [Oklab] color space.
///
/// Its components are `[L, C, h]` with
/// - `L` - the lightness as in [`Oklab`];
/// - `C` - the chromatic intensity, the natural lower bound of 0 being achromatic, usually not
///    exceeding 0.5; and
/// - `h` - the hue angle in degrees.
#[derive(Clone, Copy, Debug)]
pub struct Oklch;

impl ColorSpace for Oklch {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Oklch);

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueThird;

    const WHITE_COMPONENTS: [f32; 3] = [1., 0., 90.];

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        lab_to_lch(Oklab::from_linear_srgb(src))
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        Oklab::to_linear_srgb(lch_to_lab(src))
    }

    fn scale_chroma([l, c, h]: [f32; 3], scale: f32) -> [f32; 3] {
        [l, c * scale, h]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Oklab>() {
            lch_to_lab(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, c, h]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 1.), c.max(0.), h]
    }
}

/// 🌌 The CIELAB color space
///
/// The CIE L\*a\*b\* color space was created in 1976 to be more perceptually
/// uniform than RGB color spaces, and is both widely used and the basis of
/// other efforts to express colors, including [FreieFarbe].
///
/// Its components are `[L, a, b]` with
/// - `L` - the lightness with a natural bound between 0 and 100, where 0 represents pure black and 100
///    represents the lightness of white;
/// - `a` - how green/red the color is; and
/// - `b` - how blue/yellow the color is.
///
/// `a` and `b` are unbounded, but are usually between -160 and 160.
///
/// The color space has poor hue linearity and hue uniformity compared with
/// [Oklab], though superior lightness uniformity. Note that the lightness
/// range differs from Oklab as well; in Oklab white has a lightness of 1.
///
/// The CIE L\*a\*b\* color space is defined in terms of a D50 white point. For
/// conversion between color spaces with other illuminants (especially D65
/// as in sRGB), the standard Bradform linear chromatic adaptation transform
/// is used.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 9.1 ][css-sec].
///
/// Lab has a cylindrical counterpart: [Lch].
///
/// [FreieFarbe]: https://freiefarbe.de/en/
/// [css-sec]: https://www.w3.org/TR/css-color-4/#cie-lab
#[derive(Clone, Copy, Debug)]
pub struct Lab;

// Matrices computed from CSS Color 4 spec, then used `cargo clippy --fix`
// to reduce precision to f32 and add underscores.

// This is D65_to_D50 * lin_sRGB_to_XYZ, then rows scaled by 1 / D50[i].
const LAB_SRGB_TO_XYZ: [[f32; 3]; 3] = [
    [0.452_211_65, 0.399_412_24, 0.148_376_09],
    [0.222_493_17, 0.716_887, 0.060_619_81],
    [0.016_875_342, 0.117_659_41, 0.865_465_2],
];

// This is XYZ_to_lin_sRGB * D50_to_D65, then columns scaled by D50[i].
const LAB_XYZ_TO_SRGB: [[f32; 3]; 3] = [
    [3.022_233_7, -1.617_386, -0.404_847_65],
    [-0.943_848_25, 1.916_254_4, 0.027_593_868],
    [0.069_386_27, -0.228_976_76, 1.159_590_5],
];

const EPSILON: f32 = 216. / 24389.;
const KAPPA: f32 = 24389. / 27.;

impl ColorSpace for Lab {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Lab);

    const WHITE_COMPONENTS: [f32; 3] = [100., 0., 0.];

    fn to_linear_srgb([l, a, b]: [f32; 3]) -> [f32; 3] {
        let f1 = l * (1. / 116.) + (16. / 116.);
        let f0 = a * (1. / 500.) + f1;
        let f2 = f1 - b * (1. / 200.);
        let xyz = [f0, f1, f2].map(|value| {
            // This is EPSILON.cbrt() but that function isn't const (yet)
            const EPSILON_CBRT: f32 = 0.206_896_56;
            if value > EPSILON_CBRT {
                value * value * value
            } else {
                (116. / KAPPA) * value - (16. / KAPPA)
            }
        });
        matmul(&LAB_XYZ_TO_SRGB, xyz)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let xyz = matmul(&LAB_SRGB_TO_XYZ, src);
        let f = xyz.map(|value| {
            if value > EPSILON {
                value.cbrt()
            } else {
                (KAPPA / 116.) * value + (16. / 116.)
            }
        });
        let l = 116. * f[1] - 16.;
        let a = 500. * (f[0] - f[1]);
        let b = 200. * (f[1] - f[2]);
        [l, a, b]
    }

    fn scale_chroma([l, a, b]: [f32; 3], scale: f32) -> [f32; 3] {
        [l, a * scale, b * scale]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Lch>() {
            lab_to_lch(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, a, b]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 100.), a, b]
    }
}

/// 🌌 The cylindrical version of the [Lab] color space.
///
/// Its components are `[L, C, h]` with
/// - `L` - the lightness as in [`Lab`];
/// - `C` - the chromatic intensity, the natural lower bound of 0 being achromatic, usually not
///    exceeding 160; and
/// - `h` - the hue angle in degrees.
///
/// See [`Oklch`] for a similar color space but with better hue linearity.
#[derive(Clone, Copy, Debug)]
pub struct Lch;

impl ColorSpace for Lch {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Lch);

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueThird;

    const WHITE_COMPONENTS: [f32; 3] = [100., 0., 0.];

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        lab_to_lch(Lab::from_linear_srgb(src))
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        Lab::to_linear_srgb(lch_to_lab(src))
    }

    fn scale_chroma([l, c, h]: [f32; 3], scale: f32) -> [f32; 3] {
        [l, c * scale, h]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Lab>() {
            lch_to_lab(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, c, h]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 100.), c.max(0.), h]
    }
}

/// 🌌 The HSL color space
///
/// The HSL color space is fairly widely used and convenient, but it is
/// not based on sound color science. Among its flaws, colors with the
/// same "lightness" value can have wildly varying perceptual lightness.
///
/// Its components are `[H, S, L]` with
/// - `H` - the hue angle in degrees, with red at 0, green at 120, and blue at 240.
/// - `S` - the saturation, where 0 is gray and 100 is maximally saturated.
/// - `L` - the lightness, where 0 is black and 100 is white.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 7][css-sec].
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#the-hsl-notation
#[derive(Clone, Copy, Debug)]
pub struct Hsl;

/// Convert HSL to RGB.
///
/// Reference: § 7.1 of CSS Color 4 spec.
fn hsl_to_rgb([h, s, l]: [f32; 3]) -> [f32; 3] {
    // Don't need mod 360 for hue, it's subsumed by mod 12 below.
    let sat = s * 0.01;
    let light = l * 0.01;
    let a = sat * light.min(1.0 - light);
    [0.0, 8.0, 4.0].map(|n| {
        let x = n + h * (1.0 / 30.0);
        let k = x - 12.0 * (x * (1.0 / 12.0)).floor();
        light - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    })
}

/// Convert RGB to HSL.
///
/// Reference: § 7.2 of CSS Color 4 spec.
///
/// See <https://github.com/w3c/csswg-drafts/issues/10695> for an
/// explanation of why `hue_hack` is needed.
fn rgb_to_hsl([r, g, b]: [f32; 3], hue_hack: bool) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut hue = 0.0;
    let mut sat = 0.0;
    let light = 0.5 * (min + max);
    let d = max - min;

    const EPSILON: f32 = 1e-6;
    if d > EPSILON {
        let denom = light.min(1.0 - light);
        if denom.abs() > EPSILON {
            sat = (max - light) / denom;
        }
        hue = if max == r {
            (g - b) / d
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            // max == b
            (r - g) / d + 4.0
        };
        hue *= 60.0;
        // Deal with negative saturation from out of gamut colors
        if hue_hack && sat < 0.0 {
            hue += 180.0;
            sat = sat.abs();
        }
        hue -= 360. * (hue * (1.0 / 360.0)).floor();
    }
    [hue, sat * 100.0, light * 100.0]
}

impl ColorSpace for Hsl {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Hsl);

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueFirst;

    const WHITE_COMPONENTS: [f32; 3] = [0., 0., 100.];

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let rgb = Srgb::from_linear_srgb(src);
        rgb_to_hsl(rgb, true)
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let rgb = hsl_to_rgb(src);
        Srgb::to_linear_srgb(rgb)
    }

    fn scale_chroma([h, s, l]: [f32; 3], scale: f32) -> [f32; 3] {
        [h, s * scale, l]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Srgb>() {
            hsl_to_rgb(src)
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Hwb>() {
            rgb_to_hwb(hsl_to_rgb(src))
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([h, s, l]: [f32; 3]) -> [f32; 3] {
        [h, s.max(0.), l.clamp(0., 100.)]
    }
}

/// 🌌 The HWB color space
///
/// The HWB color space is a convenient way to represent colors. It corresponds
/// closely to popular color pickers, both a triangle with white, black, and
/// fully saturated color at the corner, and also a rectangle with a hue spectrum
/// at the top and black at the bottom, with whiteness as a separate slider. It
/// was proposed in [HWB–A More Intuitive Hue-Based Color Model].
///
/// Its components are `[H, W, B]` with
/// - `H` - the hue angle in degrees, with red at 0, green at 120, and blue at 240.
/// - `W` - an amount of whiteness to mix in, with 100 being white.
/// - `B` - an amount of blackness to mix in, with 100 being black.
///
/// The hue angle is the same as in [Hsl], and thus has the same flaw of poor hue
/// uniformity.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 8][css-sec].
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#the-hwb-notation
/// [HWB–A More Intuitive Hue-Based Color Model]: http://alvyray.com/Papers/CG/HWB_JGTv208.pdf
#[derive(Clone, Copy, Debug)]
pub struct Hwb;

/// Convert HWB to RGB.
///
/// Reference: § 8.1 of CSS Color 4 spec.
fn hwb_to_rgb([h, w, b]: [f32; 3]) -> [f32; 3] {
    let white = w * 0.01;
    let black = b * 0.01;
    if white + black >= 1.0 {
        let gray = white / (white + black);
        [gray, gray, gray]
    } else {
        let rgb = hsl_to_rgb([h, 100., 50.]);
        rgb.map(|x| white + x * (1.0 - white - black))
    }
}

/// Convert RGB to HWB.
///
/// Reference: § 8.2 of CSS Color 4 spec.
fn rgb_to_hwb([r, g, b]: [f32; 3]) -> [f32; 3] {
    let hsl = rgb_to_hsl([r, g, b], false);
    let white = r.min(g).min(b);
    let black = 1.0 - r.max(g).max(b);
    [hsl[0], white * 100., black * 100.]
}

impl ColorSpace for Hwb {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Hwb);

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueFirst;

    const WHITE_COMPONENTS: [f32; 3] = [0., 100., 0.];

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let rgb = Srgb::from_linear_srgb(src);
        rgb_to_hwb(rgb)
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let rgb = hwb_to_rgb(src);
        Srgb::to_linear_srgb(rgb)
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Srgb>() {
            hwb_to_rgb(src)
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Hsl>() {
            rgb_to_hsl(hwb_to_rgb(src), true)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([h, w, b]: [f32; 3]) -> [f32; 3] {
        [h, w.clamp(0., 100.), b.clamp(0., 100.)]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        A98Rgb, ColorSpace, DisplayP3, Hsl, Hwb, Lab, Lch, LinearSrgb, Oklab, Oklch, OpaqueColor,
        ProphotoRgb, Rec2020, Srgb, XyzD50, XyzD65,
    };

    #[must_use]
    fn almost_equal<CS: ColorSpace>(col1: [f32; 3], col2: [f32; 3]) -> bool {
        OpaqueColor::<CS>::new(col1).difference(OpaqueColor::new(col2)) < 1e-4
    }

    #[test]
    fn white_components() {
        fn check_white<CS: ColorSpace>() {
            assert!(almost_equal::<Srgb>(
                Srgb::WHITE_COMPONENTS,
                CS::convert::<Srgb>(CS::WHITE_COMPONENTS),
            ));
            assert!(almost_equal::<CS>(
                CS::WHITE_COMPONENTS,
                Srgb::convert::<CS>(Srgb::WHITE_COMPONENTS),
            ));
        }

        check_white::<A98Rgb>();
        check_white::<DisplayP3>();
        check_white::<Hsl>();
        check_white::<Hwb>();
        check_white::<Lab>();
        check_white::<Lch>();
        check_white::<LinearSrgb>();
        check_white::<Oklab>();
        check_white::<Oklch>();
        check_white::<ProphotoRgb>();
        check_white::<Rec2020>();
        check_white::<XyzD50>();
        check_white::<XyzD65>();
    }

    #[test]
    fn a98rgb_srgb() {
        for (srgb, a98) in [
            ([0.1, 0.2, 0.3], [0.155_114, 0.212_317, 0.301_498]),
            ([0., 1., 0.], [0.564_972, 1., 0.234_424]),
        ] {
            assert!(almost_equal::<Srgb>(srgb, A98Rgb::convert::<Srgb>(a98)));
            assert!(almost_equal::<A98Rgb>(a98, Srgb::convert::<A98Rgb>(srgb)));
        }
    }

    #[test]
    fn prophotorgb_srgb() {
        for (srgb, prophoto) in [
            ([0.1, 0.2, 0.3], [0.133136, 0.147659, 0.223581]),
            ([0., 1., 0.], [0.540282, 0.927599, 0.304566]),
        ] {
            assert!(almost_equal::<Srgb>(
                srgb,
                ProphotoRgb::convert::<Srgb>(prophoto)
            ));
            assert!(almost_equal::<ProphotoRgb>(
                prophoto,
                Srgb::convert::<ProphotoRgb>(srgb)
            ));
        }
    }

    #[test]
    fn rec2020_srgb() {
        for (srgb, rec2020) in [
            ([0.1, 0.2, 0.3], [0.091284, 0.134169, 0.230056]),
            ([0.05, 0.1, 0.15], [0.029785, 0.043700, 0.083264]),
            ([0., 1., 0.], [0.567542, 0.959279, 0.268969]),
        ] {
            assert!(almost_equal::<Srgb>(
                srgb,
                Rec2020::convert::<Srgb>(rec2020)
            ));
            assert!(almost_equal::<Rec2020>(
                rec2020,
                Srgb::convert::<Rec2020>(srgb)
            ));
        }
    }
}
