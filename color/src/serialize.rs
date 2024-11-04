// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CSS-compatible string serializations of colors.

use core::fmt::{Formatter, Result};

use crate::{ColorspaceTag, CssColor};

fn write_scaled_component(
    color: &CssColor,
    ix: usize,
    f: &mut Formatter<'_>,
    scale: f32,
) -> Result {
    if color.missing.contains(ix) {
        // According to the serialization rules (§15.2), missing should be converted to 0.
        // However, it seems useful to preserve these. Perhaps we want to talk about whether
        // we want string formatting to strictly follow the serialization spec.

        write!(f, "none")
    } else {
        write!(f, "{}", color.components[ix] * scale)
    }
}

fn write_modern_function(color: &CssColor, name: &str, f: &mut Formatter<'_>) -> Result {
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

fn write_color_function(color: &CssColor, name: &str, f: &mut Formatter<'_>) -> Result {
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

impl core::fmt::Display for CssColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.cs {
            ColorspaceTag::Srgb => {
                // A case can be made this isn't the best serialization in general,
                // because CSS parsing of out-of-gamut components will clamp.
                let opt_a = if self.components[3] < 1.0 { "a" } else { "" };
                write!(f, "rgb{opt_a}(")?;
                write_scaled_component(self, 0, f, 255.0)?;
                write!(f, ", ")?;
                write_scaled_component(self, 1, f, 255.0)?;
                write!(f, ", ")?;
                write_scaled_component(self, 2, f, 255.0)?;
                if self.components[3] < 1.0 {
                    write!(f, ", ")?;
                    // TODO: clamp negative values
                    write_scaled_component(self, 3, f, 1.0)?;
                }
                write!(f, ")")
            }
            ColorspaceTag::LinearSrgb => write_color_function(self, "srgb-linear", f),
            ColorspaceTag::DisplayP3 => write_color_function(self, "display-p3", f),
            ColorspaceTag::XyzD65 => write_color_function(self, "xyz", f),
            ColorspaceTag::Lab => write_modern_function(self, "lab", f),
            ColorspaceTag::Lch => write_modern_function(self, "lch", f),
            ColorspaceTag::Oklab => write_modern_function(self, "oklab", f),
            ColorspaceTag::Oklch => write_modern_function(self, "oklch", f),
            _ => todo!(),
        }
    }
}
