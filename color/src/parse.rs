// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Parse CSS4 color

use core::f64;
use core::str::FromStr;

use crate::{AlphaColor, ColorSpaceTag, DynamicColor, Missing, Srgb};

// TODO: proper error type, maybe include string offset
/// Error type for parse errors.
///
/// Currently just a static string, but likely will be changed to
/// an enum.
///
/// Discussion question: should it also contain a string offset?
pub type Error = &'static str;

#[derive(Default)]
struct Parser<'a> {
    s: &'a str,
    ix: usize,
}

/// A parsed value.
#[derive(Debug, Clone)]
enum Value<'a> {
    Symbol(&'a str),
    Number(f64),
    Percent(f64),
    Dimension(f64, &'a str),
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "deliberate choice of f32 for colors"
)]
fn color_from_components(components: [Option<f64>; 4], cs: ColorSpaceTag) -> DynamicColor {
    let mut missing = Missing::default();
    for (i, component) in components.iter().enumerate() {
        if component.is_none() {
            missing.insert(i);
        }
    }
    DynamicColor {
        cs,
        missing,
        components: components.map(|x| x.unwrap_or(0.0) as f32),
    }
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        let ix = 0;
        Parser { s, ix }
    }

    // This will be called at the start of most tokens.
    fn consume_comments(&mut self) -> Result<(), Error> {
        while self.s[self.ix..].starts_with("/*") {
            if let Some(i) = self.s[self.ix + 2..].find("*/") {
                self.ix += i + 4;
            } else {
                return Err("unclosed comment");
            }
        }
        Ok(())
    }

    fn number(&mut self) -> Option<f64> {
        self.consume_comments().ok()?;
        let tail = &self.s[self.ix..];
        let mut i = 0;
        let mut valid = false;
        if matches!(tail.as_bytes().first(), Some(b'+' | b'-')) {
            i += 1;
        }
        while let Some(c) = tail.as_bytes().get(i) {
            if c.is_ascii_digit() {
                valid = true;
                i += 1;
            } else {
                break;
            }
        }
        if let Some(b'.') = tail.as_bytes().get(i) {
            if let Some(c) = tail.as_bytes().get(i + 1) {
                if c.is_ascii_digit() {
                    valid = true;
                    i += 2;
                    while let Some(c2) = tail.as_bytes().get(i) {
                        if c2.is_ascii_digit() {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        if matches!(tail.as_bytes().get(i), Some(b'e' | b'E')) {
            let mut j = i + 1;
            if matches!(tail.as_bytes().get(j), Some(b'+' | b'-')) {
                j += 1;
            }
            if let Some(c) = tail.as_bytes().get(j) {
                if c.is_ascii_digit() {
                    i = j + 1;
                    while let Some(c2) = tail.as_bytes().get(i) {
                        if c2.is_ascii_digit() {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        if valid {
            // For this parse to fail would be strange, but we'll be careful.
            if let Ok(value) = tail[..i].parse() {
                self.ix += i;
                return Some(value);
            }
        }
        None
    }

    // Complies with ident-token production with three exceptions:
    // Escapes are not supported.
    // Non-ASCII characters are not supported.
    // Result is case sensitive.
    fn ident(&mut self) -> Option<&'a str> {
        // This does *not* strip initial whitespace.
        let tail = &self.s[self.ix..];
        let i_init = 0; // This exists as a vestige for syntax like :ident
        let mut i = i_init;
        while i < tail.len() {
            let b = tail.as_bytes()[i];
            if b.is_ascii_alphabetic()
                || b == b'_'
                || b == b'-'
                || ((i >= 2 || i == 1 && tail.as_bytes()[i_init] != b'-') && b.is_ascii_digit())
            {
                i += 1;
            } else {
                break;
            }
        }
        // Reject '', '-', and anything starting with '--'
        let mut j = i_init;
        while j < i.min(i_init + 2) {
            if tail.as_bytes()[j] == b'-' {
                j += 1;
            } else {
                self.ix += i;
                return Some(&tail[..i]);
            }
        }
        None
    }

    fn ch(&mut self, ch: u8) -> bool {
        if self.consume_comments().is_err() {
            return false;
        }
        self.raw_ch(ch)
    }

    fn raw_ch(&mut self, ch: u8) -> bool {
        if self.s[self.ix..].as_bytes().first() == Some(&ch) {
            self.ix += 1;
            true
        } else {
            false
        }
    }

    fn ws_one(&mut self) -> bool {
        if self.consume_comments().is_err() {
            return false;
        }
        let tail = &self.s[self.ix..];
        let mut i = 0;
        while let Some(&b) = tail.as_bytes().get(i) {
            if !(b == b' ' || b == b'\t' || b == b'\r' || b == b'\n') {
                break;
            }
            i += 1;
        }
        self.ix += i;
        i > 0
    }

    fn ws(&mut self) -> bool {
        if !self.ws_one() {
            return false;
        }
        while self.consume_comments().is_ok() {
            if !self.ws_one() {
                break;
            }
        }
        true
    }

    fn value(&mut self) -> Option<Value<'a>> {
        if let Some(number) = self.number() {
            if self.raw_ch(b'%') {
                Some(Value::Percent(number))
            } else if let Some(unit) = self.ident() {
                Some(Value::Dimension(number, unit))
            } else {
                Some(Value::Number(number))
            }
        } else {
            self.ident().map(Value::Symbol)
        }
    }

    /// Parse a color component.
    fn scaled_component(&mut self, scale: f64, pct_scale: f64) -> Result<Option<f64>, Error> {
        self.ws();
        let value = self.value();
        match value {
            Some(Value::Number(n)) => Ok(Some(n * scale)),
            Some(Value::Percent(n)) => Ok(Some(n * pct_scale)),
            Some(Value::Symbol("none")) => Ok(None),
            _ => Err("unknown color component"),
        }
    }

    fn angle(&mut self) -> Result<Option<f64>, Error> {
        self.ws();
        let value = self.value();
        match value {
            Some(Value::Number(n)) => Ok(Some(n)),
            Some(Value::Symbol("none")) => Ok(None),
            Some(Value::Dimension(n, dim)) => {
                let scale = match dim {
                    "deg" => 1.0,
                    "rad" => 180.0 / f64::consts::PI,
                    "grad" => 0.9,
                    "turn" => 360.0,
                    _ => return Err("unknown angle dimension"),
                };
                Ok(Some(n * scale))
            }
            _ => Err("unknown angle"),
        }
    }

    fn optional_comma(&mut self, comma: bool) -> Result<(), Error> {
        self.ws();
        if comma && !self.ch(b',') {
            Err("expected comma to separate components")
        } else {
            Ok(())
        }
    }

    fn opacity_separator(&mut self, comma: bool) -> bool {
        self.ws();
        self.ch(if comma { b',' } else { b'/' })
    }

    fn rgb(&mut self) -> Result<DynamicColor, Error> {
        if !self.raw_ch(b'(') {
            return Err("expected arguments");
        }
        // TODO: in legacy mode, be stricter about not mixing numbers
        // and percentages, and disallowing "none"
        let r = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        self.ws();
        let comma = self.ch(b',');
        let g = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        self.optional_comma(comma)?;
        let b = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        let mut alpha = Some(1.0);
        if self.opacity_separator(comma) {
            alpha = self.scaled_component(1., 0.01)?.map(|a| a.clamp(0., 1.));
        }
        self.ws();
        if !self.ch(b')') {
            return Err("expected closing parenthesis");
        }
        Ok(color_from_components([r, g, b, alpha], ColorSpaceTag::Srgb))
    }

    fn optional_alpha(&mut self) -> Result<Option<f64>, Error> {
        let mut alpha = Some(1.0);
        self.ws();
        if self.ch(b'/') {
            alpha = self.scaled_component(1., 0.01)?;
        }
        self.ws();
        Ok(alpha)
    }

    fn lab(&mut self, lmax: f64, c: f64, tag: ColorSpaceTag) -> Result<DynamicColor, Error> {
        if !self.raw_ch(b'(') {
            return Err("expected arguments");
        }
        let l = self
            .scaled_component(1., 0.01 * lmax)?
            .map(|x| x.clamp(0., lmax));
        let a = self.scaled_component(1., c)?;
        let b = self.scaled_component(1., c)?;
        let alpha = self.optional_alpha()?;
        if !self.ch(b')') {
            return Err("expected closing parenthesis");
        }
        Ok(color_from_components([l, a, b, alpha], tag))
    }

    fn lch(&mut self, lmax: f64, c: f64, tag: ColorSpaceTag) -> Result<DynamicColor, Error> {
        if !self.raw_ch(b'(') {
            return Err("expected arguments");
        }
        let l = self
            .scaled_component(1., 0.01 * lmax)?
            .map(|x| x.clamp(0., lmax));
        let c = self.scaled_component(1., c)?.map(|x| x.max(0.));
        let h = self.angle()?;
        let alpha = self.optional_alpha()?;
        if !self.ch(b')') {
            return Err("expected closing parenthesis");
        }
        Ok(color_from_components([l, c, h, alpha], tag))
    }

    fn color(&mut self) -> Result<DynamicColor, Error> {
        if !self.raw_ch(b'(') {
            return Err("expected arguments");
        }
        self.ws();
        let Some(id) = self.ident() else {
            return Err("expected identifier for colorspace");
        };
        let cs = match id {
            "srgb" => ColorSpaceTag::Srgb,
            "srgb-linear" => ColorSpaceTag::LinearSrgb,
            "display-p3" => ColorSpaceTag::DisplayP3,
            "xyz" | "xyz-d65" => ColorSpaceTag::XyzD65,
            _ => return Err("unknown colorspace"),
        };
        let r = self.scaled_component(1., 0.01)?;
        let g = self.scaled_component(1., 0.01)?;
        let b = self.scaled_component(1., 0.01)?;
        let alpha = self.optional_alpha()?;
        if !self.ch(b')') {
            return Err("expected closing parenthesis");
        }
        Ok(color_from_components([r, g, b, alpha], cs))
    }
}

// Arguably this should be an implementation of FromStr.
/// Parse a color string in CSS syntax into a color.
///
/// # Errors
///
/// Tries to return a suitable error for any invalid string, but may be
/// a little lax on some details.
pub fn parse_color(s: &str) -> Result<DynamicColor, Error> {
    if let Some(stripped) = s.strip_prefix('#') {
        let color = color_from_4bit_hex(get_4bit_hex_channels(stripped)?);
        return Ok(DynamicColor::from_alpha_color(color));
    }
    // TODO: the named x11 colors (steal from peniko)
    let mut parser = Parser::new(s);
    if let Some(id) = parser.ident() {
        match id {
            "rgb" | "rgba" => parser.rgb(),
            "lab" => parser.lab(100.0, 1.25, ColorSpaceTag::Lab),
            "lch" => parser.lch(100.0, 1.25, ColorSpaceTag::Lch),
            "oklab" => parser.lab(1.0, 0.004, ColorSpaceTag::Oklab),
            "oklch" => parser.lch(1.0, 0.004, ColorSpaceTag::Oklch),
            "transparent" => Ok(color_from_components([Some(0.); 4], ColorSpaceTag::Srgb)),
            "color" => parser.color(),
            _ => {
                if let Some([r, g, b, a]) = crate::x11_colors::lookup_palette(id) {
                    let color = AlphaColor::from_rgba8(r, g, b, a);
                    Ok(DynamicColor::from_alpha_color(color))
                } else {
                    Err("unknown color identifier")
                }
            }
        }
        // TODO: should we validate that the parser is at eof?
    } else {
        Err("unknown color syntax")
    }
}

const fn get_4bit_hex_channels(hex_str: &str) -> Result<[u8; 8], Error> {
    let mut four_bit_channels = match *hex_str.as_bytes() {
        [r, g, b] => [r, r, g, g, b, b, b'f', b'f'],
        [r, g, b, a] => [r, r, g, g, b, b, a, a],
        [r0, r1, g0, g1, b0, b1] => [r0, r1, g0, g1, b0, b1, b'f', b'f'],
        [r0, r1, g0, g1, b0, b1, a0, a1] => [r0, r1, g0, g1, b0, b1, a0, a1],
        _ => return Err("wrong number of hex digits"),
    };

    // convert to hex in-place
    // this is written without a for loop to satisfy `const`
    let mut i = 0;
    while i < four_bit_channels.len() {
        let ascii = four_bit_channels[i];
        let as_hex = match hex_from_ascii_byte(ascii) {
            Ok(hex) => hex,
            Err(e) => return Err(e),
        };
        four_bit_channels[i] = as_hex;
        i += 1;
    }
    Ok(four_bit_channels)
}

const fn hex_from_ascii_byte(b: u8) -> Result<u8, Error> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err("invalid hex digit"),
    }
}

const fn color_from_4bit_hex(components: [u8; 8]) -> AlphaColor<Srgb> {
    let [r0, r1, g0, g1, b0, b1, a0, a1] = components;
    AlphaColor::from_rgba8(r0 << 4 | r1, g0 << 4 | g1, b0 << 4 | b1, a0 << 4 | a1)
}

impl FromStr for ColorSpaceTag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "srgb" => Ok(Self::Srgb),
            "srgb-linear" => Ok(Self::LinearSrgb),
            "lab" => Ok(Self::Lab),
            "lch" => Ok(Self::Lch),
            "oklab" => Ok(Self::Oklab),
            "oklch" => Ok(Self::Oklch),
            "display-p3" => Ok(Self::DisplayP3),
            "xyz" | "xyz-d65" => Ok(Self::XyzD65),
            _ => Err("unknown colorspace name"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DynamicColor;

    use super::parse_color;

    fn assert_close_color(c1: DynamicColor, c2: DynamicColor) {
        const EPSILON: f32 = 1e-4;
        assert_eq!(c1.cs, c2.cs);
        for i in 0..4 {
            assert!((c1.components[i] - c2.components[i]).abs() < EPSILON);
        }
    }

    #[test]
    fn x11_color_names() {
        let red = parse_color("red").unwrap();
        assert_close_color(red, parse_color("rgb(255, 0, 0)").unwrap());
        let lgy = parse_color("lightgoldenrodyellow").unwrap();
        assert_close_color(lgy, parse_color("rgb(250, 250, 210)").unwrap());
        let transparent = parse_color("transparent").unwrap();
        assert_close_color(transparent, parse_color("rgba(0, 0, 0, 0)").unwrap());
    }
}
