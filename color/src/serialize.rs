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
        if self.flags.named() {
            if let Some(color_name) = self.flags.color_name() {
                return write!(f, "{}", color_name);
            }

            match self.cs {
                ColorSpaceTag::Srgb => write_legacy_function(self, "rgb", 255.0, f),
                ColorSpaceTag::Hsl | ColorSpaceTag::Hwb => {
                    let srgb = self.convert(ColorSpaceTag::Srgb);
                    write_legacy_function(&srgb, "rgb", 255.0, f)
                }
                ColorSpaceTag::Lab => write_modern_function(self, "lab", f),
                ColorSpaceTag::Lch => write_modern_function(self, "lch", f),
                ColorSpaceTag::Oklab => write_modern_function(self, "oklab", f),
                ColorSpaceTag::Oklch => write_modern_function(self, "oklch", f),
                _ => unreachable!(),
            }
        } else {
            let color_space = match self.cs {
                ColorSpaceTag::Srgb => "srgb",
                ColorSpaceTag::LinearSrgb => "srgb-linear",
                ColorSpaceTag::DisplayP3 => "display-p3",
                ColorSpaceTag::A98Rgb => "a98-rgb",
                ColorSpaceTag::ProphotoRgb => "prophoto-rgb",
                ColorSpaceTag::Rec2020 => "rec2020",
                ColorSpaceTag::AcesCg => "--acescg",
                ColorSpaceTag::Hsl => "hsl",
                ColorSpaceTag::Hwb => "hwb",
                ColorSpaceTag::XyzD50 => "xyz-d50",
                ColorSpaceTag::XyzD65 => "xyz",
                ColorSpaceTag::Lab => "lab",
                ColorSpaceTag::Lch => "lch",
                ColorSpaceTag::Oklab => "oklab",
                ColorSpaceTag::Oklch => "oklch",
            };
            write_color_function(self, color_space, f)
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
    use crate::{parse_color, Srgb};

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
            ("rgb(1,1,1)", "rgb(1, 1, 1)"),
            // TODO: output rounding? Otherwise the tests should check for approximate equality
            // (and not string equality) for these conversion cases
            (
                "hwb(740deg 20% 30% / 50%)",
                "rgba(178.5, 93.50008, 50.999996, 0.5)",
            ),
            // the next two currently fail, but should succeed (ASCII uppercase codepoints should
            // be lowercased)
            // ("ReD", "red"),
            // ("RgB(1,1,1)", "rgb(1, 1, 1)"),
            // currently fails, but should succeed (values should be clamped at parse-time)
            // ("rgb(1.1,1,1)", "rgb(1, 1, 1)"),
            ("color(srgb 1.0 1.0 1.0)", "color(srgb 1 1 1)"),
        ] {
            let result = format!("{}", parse_color(specified).unwrap());
            assert_eq!(
                result,
                expected,
                "Failed serializing specified color `{specified}`. Expected: `{expected}`. Got: `{result}`."
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
