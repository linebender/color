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

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

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

    const WHITE_COMPONENTS: [f32; 3] = [1., 1., 1.];

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

/// 🌌 The ACES2065-1 color space.
///
/// This is a linear color space with a very wide gamut. It is is often used for archival and
/// interchange.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the components are
/// `[-65504.0, 65504.0]`.
///
/// This color space is [characterized by the Academy Color Encoding System][aces20651] and is
/// specified in [SMPTE ST 2065-1:2021][smpte].
///
/// ACES2065-1 has a reference white [near D60][aceswp]; see the [XYZ-D65 color space](`XyzD65`)
/// documentation for some background information on the meaning of "reference white."
///
/// See also [`AcesCg`].
///
/// [aces20651]: https://draftdocs.acescentral.com/specifications/encodings/aces2065-1/
/// [smpte]: https://pub.smpte.org/doc/st2065-1/20200909-pub/st2065-1-2021.pdf
/// [aceswp]: https://docs.acescentral.com/tb/white-point
#[derive(Clone, Copy, Debug)]
pub struct Aces2065_1;

impl ColorSpace for Aces2065_1 {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Aces2065_1);

    const WHITE_COMPONENTS: [f32; 3] = [1.0, 1.0, 1.0];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * ACESwp_to_D65 * ACES2065_1_to_XYZ
        const ACES2065_1_TO_LINEAR_SRGB: [[f32; 3]; 3] = [
            [2.521_686, -1.134_131, -0.387_555_2],
            [-0.276_479_9, 1.372_719, -0.096_239_17],
            [-0.015_378_065, -0.152_975_34, 1.168_353_4],
        ];
        matmul(&ACES2065_1_TO_LINEAR_SRGB, src)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // XYZ_to_ACES2065_1 * D65_to_ACESwp * lin_sRGB_to_XYZ
        const LINEAR_SRGB_TO_ACES2065_1: [[f32; 3]; 3] = [
            [0.439_632_98, 0.382_988_7, 0.177_378_33],
            [0.089_776_44, 0.813_439_4, 0.096_784_13],
            [0.017_541_17, 0.111_546_55, 0.870_912_25],
        ];
        matmul(&LINEAR_SRGB_TO_ACES2065_1, src)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [
            r.clamp(-65504., 65504.),
            g.clamp(-65504., 65504.),
            b.clamp(-65504., 65504.),
        ]
    }
}

/// 🌌 The ACEScg color space.
///
/// The ACEScg color space is a linear color space. The wide gamut makes this color space useful as
/// a working space for computer graphics.
///
/// Its components are `[r, g, b]` (red, green, and blue channels respectively), with `[0, 0, 0]`
/// pure black and `[1, 1, 1]` white. The natural bounds of the components are
/// `[-65504.0, 65504.0]`, though it is unusual to clip in this color space.
///
/// This color space is defined by the Academy Color Encoding System [specification][acescg].
///
/// ACEScg has a reference white [near D60][aceswp]; see the [XYZ-D65 color space](`XyzD65`)
/// documentation for some background information on the meaning of "reference white."
///
/// See also [`Aces2065_1`].
///
/// [acescg]: https://docs.acescentral.com/specifications/acescg/
/// [aceswp]: https://docs.acescentral.com/tb/white-point
#[derive(Clone, Copy, Debug)]
pub struct AcesCg;

impl ColorSpace for AcesCg {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::AcesCg);

    const WHITE_COMPONENTS: [f32; 3] = [1.0, 1.0, 1.0];

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // XYZ_to_lin_sRGB * ACESwp_to_D65 * ACEScg_to_XYZ
        const ACESCG_TO_LINEAR_SRGB: [[f32; 3]; 3] = [
            [1.705_051, -0.621_792_14, -0.083_258_875],
            [-0.130_256_41, 1.140_804_8, -0.010_548_319],
            [-0.024_003_357, -0.128_968_97, 1.152_972_3],
        ];
        matmul(&ACESCG_TO_LINEAR_SRGB, src)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        // XYZ_to_ACEScg * D65_to_ACESwp * lin_sRGB_to_XYZ
        const LINEAR_SRGB_TO_ACESCG: [[f32; 3]; 3] = [
            [0.613_097_4, 0.339_523_14, 0.047_379_453],
            [0.070_193_72, 0.916_353_9, 0.013_452_399],
            [0.020_615_593, 0.109_569_77, 0.869_814_63],
        ];
        matmul(&LINEAR_SRGB_TO_ACESCG, src)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [
            r.clamp(-65504., 65504.),
            g.clamp(-65504., 65504.),
            b.clamp(-65504., 65504.),
        ]
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

    const WHITE_COMPONENTS: [f32; 3] = [3457. / 3585., 1., 986. / 1195.];

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

    const WHITE_COMPONENTS: [f32; 3] = [3127. / 3290., 1., 3583. / 3290.];

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
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Okhsv>() {
            Okhsv::from_oklab(src)
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, a, b]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 1.), a, b]
    }
}

impl Oklab {
    /// Find the maximum saturation S = C / L given hue (a,b) that fits in sRGB's natural gamut.
    ///
    /// a and b must be normalized such that a^2 + b^2 = 1.
    fn compute_max_srgb_saturation(a: f32, b: f32) -> f32 {
        let (k0, k1, k2, k3, k4, wl, wm, ws) = if -1.88170328f32 * a - 0.80936493f32 * b > 1. {
            // Red component
            (
                (1.19086277f32),
                (1.76576728f32),
                (0.59662641f32),
                (0.75515197f32),
                (0.56771245f32),
                (4.0767416621f32),
                (-3.3077115913f32),
                (0.2309699292f32),
            )
        } else if 1.81444104f32 * a - 1.19445276f32 * b > 1. {
            // Green component
            (
                (0.73956515f32),
                (-0.45954404f32),
                (0.08285427f32),
                (0.12541070f32),
                (0.14503204f32),
                (-1.2684380046f32),
                (2.6097574011f32),
                (-0.3413193965f32),
            )
        } else {
            // Blue component
            (
                (1.35733652f32),
                (-0.00915799f32),
                (-1.15130210f32),
                (-0.50559606f32),
                (0.00692167f32),
                (-0.0041960863f32),
                (-0.7034186147f32),
                (1.7076147010f32),
            )
        };

        let saturation = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

        let k_l = 0.3963377774 * a + 0.2158037573 * b;
        let k_m = -0.1055613458 * a - 0.0638541728 * b;
        let k_s = -0.0894841775 * a - 1.2914855480 * b;

        let l_ = 1. + saturation * k_l;
        let m_ = 1. + saturation * k_m;
        let s_ = 1. + saturation * k_s;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let l_ds = 3. * k_l * l_ * l_;
        let m_ds = 3. * k_m * m_ * m_;
        let s_ds = 3. * k_s * s_ * s_;

        let l_ds2 = 6. * k_l * k_l * l_;
        let m_ds2 = 6. * k_m * k_m * m_;
        let s_ds2 = 6. * k_s * k_s * s_;

        let f = wl * l + wm * m + ws * s;
        let f1 = wl * l_ds + wm * m_ds + ws * s_ds;
        let f2 = wl * l_ds2 + wm * m_ds2 + ws * s_ds2;

        saturation - f * f1 / (f1 * f1 - 0.5 * f * f2)
    }

    /// For a given hue (a, b) computes (L_cusp, C_cusp) to be just within sRGB's natural gamut.
    ///
    /// a and b must be normalized such that a^2 + b^2 = 1.
    fn find_srgb_cusp(a: f32, b: f32) -> (f32, f32) {
        // First, find the maximum saturation (saturation S = C/L)
        let s_cusp = Oklab::compute_max_srgb_saturation(a, b);

        // Convert to linear sRGB to find the first point where at least one of r, g or b >= 1:
        let [r, g, b] = Oklab::to_linear_srgb([1., s_cusp * a, s_cusp * b]);
        // RGB rgb_at_max = oklab_to_linear_srgb({ 1, S_cusp * a, S_cusp * b });
        let l_cusp = (1. / r.max(g).max(b)).cbrt();
        // float L_cusp = cbrtf(1.f / max(max(rgb_at_max.r, rgb_at_max.g), rgb_at_max.b));
        let c_cusp = l_cusp * s_cusp;
        (l_cusp, c_cusp)
    }

    fn lightness_toe(l: f32) -> f32 {
        const K1: f32 = 0.206;
        const K2: f32 = 0.03;
        const K3: f32 = (1. + K1) / (1. + K2);

        0.5 * (K3 * l - K1 + ((K3 * l - K1).powi(2) + 4. * K2 * K3 * l).sqrt())
    }

    fn lightness_toe_inv(l_r: f32) -> f32 {
        const K1: f32 = 0.206;
        const K2: f32 = 0.03;
        const K3: f32 = (1. + K1) / (1. + K2);

        (l_r * (l_r + K1)) / (K3 * (l_r + K2))
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
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Okhsv>() {
            Okhsv::from_oklab(lch_to_lab(src))
        } else {
            let lin_rgb = Self::to_linear_srgb(src);
            TargetCS::from_linear_srgb(lin_rgb)
        }
    }

    fn clip([l, c, h]: [f32; 3]) -> [f32; 3] {
        [l.clamp(0., 1.), c.max(0.), h]
    }
}

/// 🌌 The Okhsv color space, intended to be a perceptually uniform color picker for [sRGB](Srgb).
///
/// The Okhsv color space is a cilindrical color picker for [sRGB](Srgb)'s natural gamut. It is
/// based on the [Oklab] color space, with a slightly different formulation to achieve better
/// perceptual uniformity within sRGB's natural gamut.
///
/// The Okhsv color space is described on [Björn Ottosson's blog][bjorn].
///
/// Its components are `[h, s, v]` with
/// - `h` - the hue angle in degrees, with red at approx. 29°, green at approx. 142°, and blue at
/// approx. 264°.
/// - `s` - the saturation, where 0 is gray and 1 is maximally saturated.
/// - `v` - the value, where 0 is black and 1 is white.
///
/// Note the conversions in and out of this color space are approximations.
///
/// (TODO) See also Okhsl.
///
/// [bjorn]: https://bottosson.github.io/posts/colorpicker/
//
// This is based on the reference implementation available at
// https://github.com/bottosson/bottosson.github.io/blob/f6f08b7fde9436be1f20f66cebbc739d660898fd/misc/ok_color.h
#[derive(Clone, Copy, Debug)]
pub struct Okhsv;

impl Okhsv {
    fn to_oklab([h, s, v]: [f32; 3]) -> [f32; 3] {
        const S0: f32 = 0.5;

        let (b, a) = h.to_radians().sin_cos();

        let (l_cusp, c_cusp) = Oklab::find_srgb_cusp(a, b);
        let t_max = c_cusp / (1. - l_cusp);
        let k = 1. - S0 / c_cusp * l_cusp;

        // Compute components as if the gamut is a perfect triangle.
        let l_v = 1. - S0 / (S0 + t_max - t_max * k * s) * s;
        let c_v = S0 / (S0 + t_max - t_max * k * s) * s * t_max;

        let l = v * l_v;
        let c = v * c_v;

        // Compensate for both the lightness toe and the curved top part of the triangle.
        let l_vt = Oklab::lightness_toe_inv(l_v);
        let c_vt = c_v * l_vt / l_v;

        let l_new = Oklab::lightness_toe_inv(l);
        let c = c * l_new / l;
        let l = l_new;

        let [r_scale, g_scale, b_scale] = Oklab::to_linear_srgb([l_vt, a * c_vt, b * c_vt]);
        let scale_l = (1. / r_scale.max(g_scale).max(b_scale).max(0.)).cbrt();

        let c = c * scale_l;
        [l * scale_l, a * c, b * c]
    }

    fn from_oklab([l, a, b]: [f32; 3]) -> [f32; 3] {
        const S0: f32 = 0.5;

        let c = (a * a + b * b).sqrt();
        let a_ = a / c;
        let b_ = b / c;

        let (l_cusp, c_cusp) = Oklab::find_srgb_cusp(a_, b_);
        let t_max = c_cusp / (1. - l_cusp);
        let k = 1. - S0 / c_cusp * l_cusp;

        // First compute the components first we find L_v, C_v, L_vt and C_vt
        let t = t_max / (c + l * t_max);
        let l_v = t * l;
        let c_v = t * c;

        let l_vt = Oklab::lightness_toe_inv(l_v);
        let c_vt = c_v * l_vt / l_v;

        // Invert the lightness toe and the compensation for the curved top part of the triangle.
        let [r_scale, g_scale, b_scale] = Oklab::to_linear_srgb([l_vt, a_ * c_vt, b_ * c_vt]);
        let scale_l = (1. / r_scale.max(g_scale).max(b_scale).max(0.)).cbrt();

        let l = Oklab::lightness_toe(l / scale_l);

        // Compute the cilindrical v and s.
        let v = l / l_v;
        let s = (S0 + t_max) * c_v / ((t_max * S0) + t_max * k * c_v);

        let h = f32::consts::PI + f32::atan2(-b_, -a_);
        [h.to_degrees(), s, v]
    }
}

impl ColorSpace for Okhsv {
    // const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Oklch);
    const TAG: Option<ColorSpaceTag> = None;

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueFirst;

    const WHITE_COMPONENTS: [f32; 3] = [0., 0., 1.];

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        Okhsv::from_oklab(Oklab::from_linear_srgb(src))
    }

    fn to_linear_srgb([h, s, v]: [f32; 3]) -> [f32; 3] {
        Oklab::to_linear_srgb(Self::to_oklab([h, s, v]))
    }

    fn scale_chroma([l, c, h]: [f32; 3], scale: f32) -> [f32; 3] {
        [l, c * scale, h]
    }

    fn convert<TargetCS: ColorSpace>(src: [f32; 3]) -> [f32; 3] {
        if TypeId::of::<Self>() == TypeId::of::<TargetCS>() {
            src
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Oklab>() {
            Okhsv::to_oklab(src)
        } else if TypeId::of::<TargetCS>() == TypeId::of::<Oklch>() {
            lab_to_lch(Okhsv::to_oklab(src))
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
        A98Rgb, Aces2065_1, AcesCg, ColorSpace, DisplayP3, Hsl, Hwb, Lab, Lch, LinearSrgb, Okhsv,
        Oklab, Oklch, OpaqueColor, ProphotoRgb, Rec2020, Srgb, XyzD50, XyzD65,
    };

    #[must_use]
    fn almost_equal<CS: ColorSpace>(col1: [f32; 3], col2: [f32; 3], absolute_epsilon: f32) -> bool {
        OpaqueColor::<CS>::new(col1).difference(OpaqueColor::new(col2)) <= absolute_epsilon
    }

    /// The maximal magnitude of the color components. Useful for calculating relative errors.
    fn magnitude(col: [f32; 3]) -> f32 {
        col[0].abs().max(col[1].abs()).max(col[2].abs())
    }

    #[test]
    fn roundtrip() {
        fn test_roundtrips<Source: ColorSpace, Dest: ColorSpace>(colors: &[[f32; 3]]) {
            /// A tight bound on relative numerical precision.
            const RELATIVE_EPSILON: f32 = f32::EPSILON * 16.;

            for color in colors {
                let intermediate = Source::convert::<Dest>(*color);
                let roundtripped = Dest::convert::<Source>(intermediate);

                // The roundtrip error is measured in linear sRGB. This adds more conversions, but
                // makes the components analogous.
                let linsrgb_color = Source::to_linear_srgb(*color);
                let linsrgb_roundtripped = Source::to_linear_srgb(roundtripped);

                // The absolute epsilon is based on the maximal magnitude of the source color
                // components. The magnitude is at least 1, as that is the natural bound of linear
                // sRGB channels and prevents numerical issues around 0.
                let absolute_epsilon = magnitude(linsrgb_color).max(1.) * RELATIVE_EPSILON;
                assert!(almost_equal::<LinearSrgb>(
                    linsrgb_color,
                    linsrgb_roundtripped,
                    absolute_epsilon,
                ));
            }
        }

        // Generate some values to test rectangular color spaces.
        let rectangular_values = {
            let components = [
                0., 1., -1., 0.5, 1234., -1234., 1.000_001, 0.000_001, -0.000_001,
            ];
            let mut values = Vec::new();
            for c0 in components {
                for c1 in components {
                    for c2 in components {
                        values.push([c0, c1, c2]);
                    }
                }
            }
            values
        };

        test_roundtrips::<LinearSrgb, Srgb>(&rectangular_values);
        test_roundtrips::<DisplayP3, Srgb>(&rectangular_values);
        test_roundtrips::<A98Rgb, Srgb>(&rectangular_values);
        test_roundtrips::<ProphotoRgb, Srgb>(&rectangular_values);
        test_roundtrips::<Rec2020, Srgb>(&rectangular_values);
        test_roundtrips::<Aces2065_1, Srgb>(&rectangular_values);
        test_roundtrips::<AcesCg, Srgb>(&rectangular_values);
        test_roundtrips::<XyzD50, Srgb>(&rectangular_values);
        test_roundtrips::<XyzD65, Srgb>(&rectangular_values);

        test_roundtrips::<Oklab, Srgb>(&[
            [0., 0., 0.],
            [1., 0., 0.],
            [0.2, 0.2, -0.1],
            [2.0, 0., -0.4],
        ]);
    }

    #[test]
    fn white_components() {
        fn check_white<CS: ColorSpace>() {
            assert!(almost_equal::<Srgb>(
                Srgb::WHITE_COMPONENTS,
                CS::convert::<Srgb>(CS::WHITE_COMPONENTS),
                1e-4,
            ));
            assert!(almost_equal::<CS>(
                CS::WHITE_COMPONENTS,
                Srgb::convert::<CS>(Srgb::WHITE_COMPONENTS),
                1e-4,
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
        check_white::<Aces2065_1>();
        check_white::<AcesCg>();
        check_white::<XyzD50>();
        check_white::<XyzD65>();
    }

    #[test]
    fn a98rgb_srgb() {
        for (srgb, a98) in [
            ([0.1, 0.2, 0.3], [0.155_114, 0.212_317, 0.301_498]),
            ([0., 1., 0.], [0.564_972, 1., 0.234_424]),
        ] {
            assert!(almost_equal::<Srgb>(
                srgb,
                A98Rgb::convert::<Srgb>(a98),
                1e-4
            ));
            assert!(almost_equal::<A98Rgb>(
                a98,
                Srgb::convert::<A98Rgb>(srgb),
                1e-4
            ));
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
                ProphotoRgb::convert::<Srgb>(prophoto),
                1e-4
            ));
            assert!(almost_equal::<ProphotoRgb>(
                prophoto,
                Srgb::convert::<ProphotoRgb>(srgb),
                1e-4
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
                Rec2020::convert::<Srgb>(rec2020),
                1e-4
            ));
            assert!(almost_equal::<Rec2020>(
                rec2020,
                Srgb::convert::<Rec2020>(srgb),
                1e-4
            ));
        }
    }

    #[test]
    fn aces2065_1_srgb() {
        for (srgb, aces2065_1) in [
            ([0.6, 0.5, 0.4], [0.245_59, 0.215_57, 0.145_18]),
            ([0.0, 0.5, 1.0], [0.259_35, 0.270_89, 0.894_79]),
        ] {
            assert!(almost_equal::<Srgb>(
                srgb,
                Aces2065_1::convert::<Srgb>(aces2065_1),
                1e-4
            ));
            assert!(almost_equal::<Aces2065_1>(
                aces2065_1,
                Srgb::convert::<Aces2065_1>(srgb),
                1e-4
            ));
        }
    }

    #[test]
    fn okhsv_srgb() {
        // Test against the reference implementation
        // https://github.com/bottosson/bottosson.github.io/blob/f6f08b7fde9436be1f20f66cebbc739d660898fd/misc/ok_color.h
        //
        // Note these are not exact conversion results; the reference implementation computes an
        // approximation.

        for (okhsv, srgb) in [
            ([256., 1., 1.], [-0.00010300, 0.50359923, 0.99999982]),
            ([30., 0.5, 0.25], [0.24300897, 0.12560070, 0.10679763]),
        ] {
            assert!(almost_equal::<Srgb>(
                Okhsv::convert::<Srgb>(okhsv),
                srgb,
                1e-4
            ));
        }

        dbg!(Srgb::convert::<Okhsv>(Okhsv::convert::<Srgb>([
            40., 4.5, 4.4
        ])));
        for (srgb, okhsv) in [
            ([0.6, 0.5, 0.4], [66.72554016, 0.28508663, 0.62701088]),
            ([0., 0.5, 1.], [256.21524048, 0.99996287, 0.99999970]),
        ] {
            assert!(almost_equal::<Srgb>(
                okhsv,
                Srgb::convert::<Okhsv>(srgb),
                1e-4
            ));
        }
    }
}
