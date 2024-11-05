// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::f32;

use crate::{matmul, tagged::ColorSpaceTag};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

/// The main trait for color spaces.
///
/// This can be implemented by clients for conversions in and out of
/// new color spaces. It is expected to be a zero-sized type.
///
/// The linear sRGB color space is central, and other color spaces are
/// defined as conversions in and out of that. A color space does not
/// explicitly define a gamut, so generally conversions will succeed
/// and round-trip, subject to numerical precision.
///
/// White point is not explicitly represented. For color spaces with a
/// white point other than D65 (the native white point for sRGB), use
/// a linear Bradford chromatic adaptation, following CSS Color 4.
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
}

/// The layout of a color space, particularly the hue channel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ColorSpaceLayout {
    Rectangular,
    HueFirst,
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

#[derive(Clone, Copy, Debug)]
pub struct LinearSrgb;

impl ColorSpace for LinearSrgb {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::LinearSrgb);

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
}

// It might be a better idea to write custom debug impls for AlphaColor and friends
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

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(srgb_to_lin)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(lin_to_srgb)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DisplayP3;

impl ColorSpace for DisplayP3 {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::DisplayP3);

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
}

#[derive(Clone, Copy, Debug)]
pub struct XyzD65;

impl ColorSpace for XyzD65 {
    const IS_LINEAR: bool = true;

    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::XyzD65);

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
}

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

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let lms = matmul(&OKLAB_LAB_TO_LMS, src).map(|x| x * x * x);
        matmul(&OKLAB_LMS_TO_SRGB, lms)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let lms = matmul(&OKLAB_SRGB_TO_LMS, src).map(f32::cbrt);
        matmul(&OKLAB_LMS_TO_LAB, lms)
    }

    fn scale_chroma(src: [f32; 3], scale: f32) -> [f32; 3] {
        [src[0], src[1] * scale, src[2] * scale]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Oklch;

impl ColorSpace for Oklch {
    const TAG: Option<ColorSpaceTag> = Some(ColorSpaceTag::Oklch);

    const LAYOUT: ColorSpaceLayout = ColorSpaceLayout::HueThird;

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let lab = Oklab::from_linear_srgb(src);
        let l = lab[0];
        let mut h = lab[2].atan2(lab[1]) * (180. / f32::consts::PI);
        if h < 0.0 {
            h += 360.0;
        }
        let c = lab[2].hypot(lab[1]);
        [l, c, h]
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        let l = src[0];
        let (sin, cos) = (src[2] * (f32::consts::PI / 180.)).sin_cos();
        let a = src[1] * cos;
        let b = src[1] * sin;
        Oklab::to_linear_srgb([l, a, b])
    }

    fn scale_chroma(src: [f32; 3], scale: f32) -> [f32; 3] {
        [src[0], src[1] * scale, src[2]]
    }
}
