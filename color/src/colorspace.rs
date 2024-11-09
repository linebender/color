// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{any::TypeId, f32};

use crate::{matmul, tagged::ColorSpaceTag};

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

/// The linear-light RGB color space with [sRGB](`Srgb`) primaries.
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

/// The standard RGB color space.
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

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(srgb_to_lin)
    }

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        src.map(lin_to_srgb)
    }

    fn clip([r, g, b]: [f32; 3]) -> [f32; 3] {
        [r.clamp(0., 1.), g.clamp(0., 1.), b.clamp(0., 1.)]
    }
}

/// The Display P3 color space, often used for wide-gamut displays.
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

/// The CIE XYZ color space with a 2° observer and a reference white of D65.
///
/// Its components are `[X, Y, Z]`. The components are unbounded, but are usually positive.
/// Reference white has a luminance `Y` of 1.
///
/// This corresponds to the color space in [CSS Color Module Level 4 § 10.8][css-sec]. It is
/// defined in CIE 015:2018.
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
/// [wikipedia-cie]: https://en.wikipedia.org/wiki/CIE_1931_color_space
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

    fn clip([x, y, z]: [f32; 3]) -> [f32; 3] {
        [x, y, z]
    }
}

/// The Oklab color space, intended to be a perceptually uniform color space.
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
/// defined on [Björn Ottosson's blog][bjorn]. It is similar to the CIELAB color space.
///
/// Oklab has a cylindrical counterpart: [Oklch](`Oklch`).
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#ok-lab
/// [bjorn]: https://bottosson.github.io/posts/oklab/
// todo: link to the CIELAB color space.
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

/// The cylindrical version of the [Oklab] color space.
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

    fn from_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        lab_to_lch(Oklab::from_linear_srgb(src))
    }

    fn to_linear_srgb(src: [f32; 3]) -> [f32; 3] {
        Oklab::to_linear_srgb(lch_to_lab(src))
    }

    fn scale_chroma(src: [f32; 3], scale: f32) -> [f32; 3] {
        [src[0], src[1] * scale, src[2]]
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
