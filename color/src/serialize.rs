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
    if color.flags.missing(ix) {
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
                ColorSpaceTag::Hsl => write_legacy_function(self, "hsl", 1.0, f),
                ColorSpaceTag::Hwb => write_modern_function(self, "hwb", f),
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

#[cfg(test)]
mod tests {
    use crate::parse_color;

    #[test]
    fn specified_to_serialized() {
        for (specified, expected) in [
            ("rgb(1,1,1)", "rgb(1, 1, 1)"),
            // currently fails, but should succeed (values should be clamped at parse-time)
            // ("rgb(1.1,1,1)", "rgb(1, 1, 1)"),
            ("color(srgb 1.0 1.0 1.0)", "color(srgb 1 1 1)"),
            ("rosybrown", "rosybrown"),
            ("red", "red"),
            ("transparent", "transparent"),
            ("yellowgreen", "yellowgreen"),
        ] {
            let result = format!("{}", parse_color(specified).unwrap());
            assert_eq!(
                result,
                expected,
                "Failed serializing specified color `{specified}`. Expected: `{expected}`. Got: `{result}`."
            );
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
            write!(f, "#{:02x}{:02x}{:02x})", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02x}{:02x}{:02x}{:02x})",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

impl core::fmt::UpperHex for Rgba8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.a == 255 {
            write!(f, "#{:02X}{:02X}{:02X})", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02X}{:02X}{:02X}{:02X})",
                self.r, self.g, self.b, self.a
            )
        }
    }
}
