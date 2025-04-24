// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CSS-compatible string serializations of colors.

use core::fmt::{Formatter, Result};

use crate::{ColorSpaceTag, DynamicColor, Rgba8};

fn write_scaled_component(
    color: &DynamicColor,
    ix: usize,
    f: &mut Formatter<'_>,
    scale: f32,
) -> Result {
    if color.flags.missing().contains(ix) {
        // According to the serialization rules (§15.2), missing should be converted to 0.
        // However, it seems useful to preserve these. Perhaps we want to talk about whether
        // we want string formatting to strictly follow the serialization spec.

        write!(f, "none")
    } else {
        write!(f, "{}", color.components[ix] * scale)
    }
}

fn write_modern_function(color: &DynamicColor, name: &str, f: &mut Formatter<'_>) -> Result {
    write!(f, "{name}(")?;
    write_scaled_component(color, 0, f, 1.0)?;
    write!(f, " ")?;
    write_scaled_component(color, 1, f, 1.0)?;
    write!(f, " ")?;
    write_scaled_component(color, 2, f, 1.0)?;
    if color.components[3] < 1.0 {
        write!(f, " / ")?;
        // TODO: clamp negative values
        write_scaled_component(color, 3, f, 1.0)?;
    }
    write!(f, ")")
}

fn write_color_function(color: &DynamicColor, name: &str, f: &mut Formatter<'_>) -> Result {
    write!(f, "color({name} ")?;
    write_scaled_component(color, 0, f, 1.0)?;
    write!(f, " ")?;
    write_scaled_component(color, 1, f, 1.0)?;
    write!(f, " ")?;
    write_scaled_component(color, 2, f, 1.0)?;
    if color.components[3] < 1.0 {
        write!(f, " / ")?;
        // TODO: clamp negative values
        write_scaled_component(color, 3, f, 1.0)?;
    }
    write!(f, ")")
}

fn write_legacy_function(
    color: &DynamicColor,
    name: &str,
    scale: f32,
    f: &mut Formatter<'_>,
) -> Result {
    let opt_a = if color.components[3] < 1.0 { "a" } else { "" };
    write!(f, "{name}{opt_a}(")?;
    write_scaled_component(color, 0, f, scale)?;
    write!(f, ", ")?;
    write_scaled_component(color, 1, f, scale)?;
    write!(f, ", ")?;
    write_scaled_component(color, 2, f, scale)?;
    if color.components[3] < 1.0 {
        write!(f, ", ")?;
        // TODO: clamp negative values
        write_scaled_component(color, 3, f, 1.0)?;
    }
    write!(f, ")")
}

impl core::fmt::Display for DynamicColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(color_name) = self.flags.color_name() {
            return write!(f, "{color_name}");
        }

        match self.cs {
            ColorSpaceTag::Srgb if self.flags.named() => {
                write_legacy_function(self, "rgb", 255.0, f)
            }
            ColorSpaceTag::Hsl | ColorSpaceTag::Hwb if self.flags.named() => {
                let srgb = self.convert(ColorSpaceTag::Srgb);
                write_legacy_function(&srgb, "rgb", 255.0, f)
            }
            ColorSpaceTag::Srgb => write_color_function(self, "srgb", f),
            ColorSpaceTag::LinearSrgb => write_color_function(self, "srgb-linear", f),
            ColorSpaceTag::DisplayP3 => write_color_function(self, "display-p3", f),
            ColorSpaceTag::A98Rgb => write_color_function(self, "a98-rgb", f),
            ColorSpaceTag::ProphotoRgb => write_color_function(self, "prophoto-rgb", f),
            ColorSpaceTag::Rec2020 => write_color_function(self, "rec2020", f),
            ColorSpaceTag::Aces2065_1 => write_color_function(self, "--aces2065-1", f),
            ColorSpaceTag::AcesCg => write_color_function(self, "--acescg", f),
            ColorSpaceTag::Hsl => write_legacy_function(self, "hsl", 1.0, f),
            ColorSpaceTag::Hwb => write_modern_function(self, "hwb", f),
            ColorSpaceTag::XyzD50 => write_color_function(self, "xyz-d50", f),
            ColorSpaceTag::XyzD65 => write_color_function(self, "xyz-d65", f),
            ColorSpaceTag::Lab => write_modern_function(self, "lab", f),
            ColorSpaceTag::Lch => write_modern_function(self, "lch", f),
            ColorSpaceTag::Oklab => write_modern_function(self, "oklab", f),
            ColorSpaceTag::Oklch => write_modern_function(self, "oklch", f),
        }
    }
}

impl core::fmt::Display for Rgba8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.a == 255 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            let a = self.a as f32 * (1.0 / 255.0);
            write!(f, "rgba({}, {}, {}, {a})", self.r, self.g, self.b)
        }
    }
}

impl core::fmt::LowerHex for Rgba8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

impl core::fmt::UpperHex for Rgba8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.a == 255 {
            write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02X}{:02X}{:02X}{:02X}",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::{parse_color, AlphaColor, DynamicColor, Hsl, Oklab, Srgb, XyzD65};
    use alloc::format;

    #[test]
    fn rgb8() {
        let c = parse_color("#abcdef").unwrap().to_alpha_color::<Srgb>();
        assert_eq!(format!("{:x}", c.to_rgba8()), "#abcdef");
        assert_eq!(format!("{:X}", c.to_rgba8()), "#ABCDEF");
        let c_alpha = c.with_alpha(1. / 3.);
        assert_eq!(format!("{:x}", c_alpha.to_rgba8()), "#abcdef55");
        assert_eq!(format!("{:X}", c_alpha.to_rgba8()), "#ABCDEF55");
    }

    #[test]
    fn specified_to_serialized() {
        for (specified, expected) in [
            ("#ff0000", "rgb(255, 0, 0)"),
            ("rgb(255,0,0)", "rgb(255, 0, 0)"),
            ("rgba(255,0,0,50%)", "rgba(255, 0, 0, 0.5)"),
            ("rgb(255 0 0 / 95%)", "rgba(255, 0, 0, 0.95)"),
            // TODO: output rounding? Otherwise the tests should check for approximate equality
            // (and not string equality) for these conversion cases
            // (
            //     "hwb(740deg 20% 30% / 50%)",
            //     "rgba(178.5, 93.50008, 50.999996, 0.5)",
            // ),
            ("ReD", "red"),
            ("RgB(1,1,1)", "rgb(1, 1, 1)"),
            ("rgb(257,-2,50)", "rgb(255, 0, 50)"),
            ("color(srgb 1.0 1.0 1.0)", "color(srgb 1 1 1)"),
            ("oklab(0.4 0.2 -0.2)", "oklab(0.4 0.2 -0.2)"),
            ("lab(20% 0 60)", "lab(20 0 60)"),
        ] {
            let result = format!("{}", parse_color(specified).unwrap());
            assert_eq!(
                result,
                expected,
                "Failed serializing specified color `{specified}`. Expected: `{expected}`. Got: `{result}`."
            );
        }

        // TODO: this can be removed when the "output rounding" TODO above is resolved. Here we
        // just check the prefix is as expected.
        for (specified, expected_prefix) in [
            ("hwb(740deg 20% 30%)", "rgb("),
            ("hwb(740deg 20% 30% / 50%)", "rgba("),
            ("hsl(120deg 50% 25%)", "rgb("),
            ("hsla(0.4turn 50% 25% / 50%)", "rgba("),
        ] {
            let result = format!("{}", parse_color(specified).unwrap());
            assert!(
                result.starts_with(expected_prefix),
                "Failed serializing specified color `{specified}`. Expected the serialization to start with: `{expected_prefix}`. Got: `{result}`."
            );
        }
    }

    #[test]
    fn generated_to_serialized() {
        for (color, expected) in [
            (
                DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([0.5, 0.2, 1.1, 0.5])),
                "color(srgb 0.5 0.2 1.1 / 0.5)",
            ),
            (
                DynamicColor::from_alpha_color(AlphaColor::<Oklab>::new([0.4, 0.2, -0.2, 1.])),
                "oklab(0.4 0.2 -0.2)",
            ),
            (
                DynamicColor::from_alpha_color(AlphaColor::<XyzD65>::new([
                    0.472, 0.372, 0.131, 1.,
                ])),
                "color(xyz-d65 0.472 0.372 0.131)",
            ),
            // Perhaps this should actually serialize to `rgb(...)`.
            (
                DynamicColor::from_alpha_color(AlphaColor::<Hsl>::new([120., 50., 25., 1.])),
                "hsl(120, 50, 25)",
            ),
        ] {
            let result = format!("{color}");
            assert_eq!(
                result,
                expected,
                "Failed serializing specified color `{color}`. Expected: `{expected}`. Got: `{result}`."
            );
        }
    }

    #[test]
    fn roundtrip_named_colors() {
        for name in crate::x11_colors::NAMES {
            let result = format!("{}", parse_color(name).unwrap());
            assert_eq!(
                result,
                name,
                "Failed serializing specified named color `{name}`. Expected it to roundtrip. Got: `{result}`."
            );
        }
    }
}
