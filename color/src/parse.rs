// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Parse CSS4 color

use core::error::Error;
use core::f64;
use core::fmt;
use core::str;
use core::str::FromStr;

use crate::{
    AlphaColor, ColorSpace, ColorSpaceTag, DynamicColor, Flags, Missing, OpaqueColor, PremulColor,
    Srgb,
};

// TODO: maybe include string offset
/// Error type for parse errors.
///
/// Discussion question: should it also contain a string offset?
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParseError {
    /// Unclosed comment
    UnclosedComment,
    /// Unknown angle dimension
    UnknownAngleDimension,
    /// Unknown angle
    UnknownAngle,
    /// Unknown color component
    UnknownColorComponent,
    /// Unknown color identifier
    UnknownColorIdentifier,
    /// Unknown color space
    UnknownColorSpace,
    /// Unknown color syntax
    UnknownColorSyntax,
    /// Expected arguments
    ExpectedArguments,
    /// Expected closing parenthesis
    ExpectedClosingParenthesis,
    /// Expected color space identifier
    ExpectedColorSpaceIdentifier,
    /// Expected comma
    ExpectedComma,
    /// Expected end of string
    ExpectedEndOfString,
    /// Wrong number of hex digits
    WrongNumberOfHexDigits,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match *self {
            Self::UnclosedComment => "unclosed comment",
            Self::UnknownAngleDimension => "unknown angle dimension",
            Self::UnknownAngle => "unknown angle",
            Self::UnknownColorComponent => "unknown color component",
            Self::UnknownColorIdentifier => "unknown color identifier",
            Self::UnknownColorSpace => "unknown color space",
            Self::UnknownColorSyntax => "unknown color syntax",
            Self::ExpectedArguments => "expected arguments",
            Self::ExpectedClosingParenthesis => "expected closing parenthesis",
            Self::ExpectedColorSpaceIdentifier => "expected color space identifier",
            Self::ExpectedComma => "expected comma",
            Self::ExpectedEndOfString => "expected end of string",
            Self::WrongNumberOfHexDigits => "wrong number of hex digits",
        };
        f.write_str(msg)
    }
}

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

/// Whether or not we are parsing modern or legacy mode syntax.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Legacy,
    Modern,
}

impl Mode {
    fn alpha_separator(self) -> u8 {
        match self {
            Self::Legacy => b',',
            Self::Modern => b'/',
        }
    }
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
        flags: Flags::from_missing(missing),
        components: components.map(|x| x.unwrap_or(0.0) as f32),
    }
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        let ix = 0;
        Parser { s, ix }
    }

    // This will be called at the start of most tokens.
    fn consume_comments(&mut self) -> Result<(), ParseError> {
        while self.s[self.ix..].starts_with("/*") {
            if let Some(i) = self.s[self.ix + 2..].find("*/") {
                self.ix += i + 4;
            } else {
                return Err(ParseError::UnclosedComment);
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

    /// Attempt to read the exact ASCII character given, returning whether that character was read.
    ///
    /// The parser proceeds to the next character if the character was successfully read.
    fn raw_ch(&mut self, ch: u8) -> bool {
        debug_assert!(ch.is_ascii(), "`ch` must be an ASCII character");
        if self.s.as_bytes().get(self.ix) == Some(&ch) {
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
    fn scaled_component(&mut self, scale: f64, pct_scale: f64) -> Result<Option<f64>, ParseError> {
        self.ws();
        let value = self.value();
        match value {
            Some(Value::Number(n)) => Ok(Some(n * scale)),
            Some(Value::Percent(n)) => Ok(Some(n * pct_scale)),
            Some(Value::Symbol(s)) if s.eq_ignore_ascii_case("none") => Ok(None),
            _ => Err(ParseError::UnknownColorComponent),
        }
    }

    fn angle(&mut self) -> Result<Option<f64>, ParseError> {
        self.ws();
        let value = self.value();
        match value {
            Some(Value::Number(n)) => Ok(Some(n)),
            Some(Value::Symbol(s)) if s.eq_ignore_ascii_case("none") => Ok(None),
            Some(Value::Dimension(n, dim)) => {
                let mut buf = [0; LOWERCASE_BUF_SIZE];
                let dim_lc = make_lowercase(dim, &mut buf);
                let scale = match dim_lc {
                    "deg" => 1.0,
                    "rad" => 180.0 / f64::consts::PI,
                    "grad" => 0.9,
                    "turn" => 360.0,
                    _ => return Err(ParseError::UnknownAngleDimension),
                };
                Ok(Some(n * scale))
            }
            _ => Err(ParseError::UnknownAngle),
        }
    }

    fn optional_comma(&mut self, comma: bool) -> Result<(), ParseError> {
        self.ws();
        if comma && !self.ch(b',') {
            Err(ParseError::ExpectedComma)
        } else {
            Ok(())
        }
    }

    fn rgb(&mut self) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        // TODO: in legacy mode, be stricter about not mixing numbers
        // and percentages, and disallowing "none"
        let r = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        self.ws();
        let comma = self.ch(b',');
        let mode = if comma { Mode::Legacy } else { Mode::Modern };
        let g = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        self.optional_comma(comma)?;
        let b = self
            .scaled_component(1. / 255., 0.01)?
            .map(|x| x.clamp(0., 1.));
        let alpha = self.alpha(mode)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([r, g, b, alpha], ColorSpaceTag::Srgb))
    }

    /// Read a slash separator and an alpha value.
    ///
    /// The value may be either number or a percentage.
    ///
    /// The alpha value defaults to `1.0` if not present. The value will be clamped
    /// to the range [0, 1].
    ///
    /// If the value is `"none"`, then `Ok(None)` will be returned.
    ///
    /// The separator will be a `'/'` in modern mode and a `','` in legacy mode.
    /// If no separator is present, then the default value will be returned.
    ///
    /// Reference: § 4.2 of CSS Color 4 spec.
    fn alpha(&mut self, mode: Mode) -> Result<Option<f64>, ParseError> {
        self.ws();
        if self.ch(mode.alpha_separator()) {
            Ok(self.scaled_component(1., 0.01)?.map(|a| a.clamp(0., 1.)))
        } else {
            Ok(Some(1.0))
        }
    }

    fn lab(&mut self, lmax: f64, c: f64, tag: ColorSpaceTag) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        let l = self
            .scaled_component(1., 0.01 * lmax)?
            .map(|x| x.clamp(0., lmax));
        let a = self.scaled_component(1., c)?;
        let b = self.scaled_component(1., c)?;
        let alpha = self.alpha(Mode::Modern)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([l, a, b, alpha], tag))
    }

    fn lch(&mut self, lmax: f64, c: f64, tag: ColorSpaceTag) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        let l = self
            .scaled_component(1., 0.01 * lmax)?
            .map(|x| x.clamp(0., lmax));
        let c = self.scaled_component(1., c)?.map(|x| x.max(0.));
        let h = self.angle()?;
        let alpha = self.alpha(Mode::Modern)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([l, c, h, alpha], tag))
    }

    fn hsl(&mut self) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        let h = self.angle()?;
        let comma = self.ch(b',');
        let mode = if comma { Mode::Legacy } else { Mode::Modern };
        let s = self.scaled_component(1., 1.)?.map(|x| x.max(0.));
        self.optional_comma(comma)?;
        let l = self.scaled_component(1., 1.)?;
        let alpha = self.alpha(mode)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([h, s, l, alpha], ColorSpaceTag::Hsl))
    }

    fn hwb(&mut self) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        let h = self.angle()?;
        let w = self.scaled_component(1., 1.)?;
        let b = self.scaled_component(1., 1.)?;
        let alpha = self.alpha(Mode::Modern)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([h, w, b, alpha], ColorSpaceTag::Hwb))
    }

    fn color(&mut self) -> Result<DynamicColor, ParseError> {
        if !self.raw_ch(b'(') {
            return Err(ParseError::ExpectedArguments);
        }
        self.ws();
        let Some(id) = self.ident() else {
            return Err(ParseError::ExpectedColorSpaceIdentifier);
        };
        let mut buf = [0; LOWERCASE_BUF_SIZE];
        let id_lc = make_lowercase(id, &mut buf);
        let cs = match id_lc {
            "srgb" => ColorSpaceTag::Srgb,
            "srgb-linear" => ColorSpaceTag::LinearSrgb,
            "display-p3" => ColorSpaceTag::DisplayP3,
            "a98-rgb" => ColorSpaceTag::A98Rgb,
            "prophoto-rgb" => ColorSpaceTag::ProphotoRgb,
            "rec2020" => ColorSpaceTag::Rec2020,
            "xyz-d50" => ColorSpaceTag::XyzD50,
            "xyz" | "xyz-d65" => ColorSpaceTag::XyzD65,
            _ => return Err(ParseError::UnknownColorSpace),
        };
        let r = self.scaled_component(1., 0.01)?;
        let g = self.scaled_component(1., 0.01)?;
        let b = self.scaled_component(1., 0.01)?;
        let alpha = self.alpha(Mode::Modern)?;
        self.ws();
        if !self.ch(b')') {
            return Err(ParseError::ExpectedClosingParenthesis);
        }
        Ok(color_from_components([r, g, b, alpha], cs))
    }
}

/// Parse a color string prefix in CSS syntax into a color.
///
/// Returns the byte offset of the unparsed remainder of the string and the parsed color. See also
/// [`parse_color`].
///
/// # Errors
///
/// Tries to return a suitable error for any invalid string, but may be
/// a little lax on some details.
pub fn parse_color_prefix(s: &str) -> Result<(usize, DynamicColor), ParseError> {
    #[inline]
    fn set_from_named_color_space(mut color: DynamicColor) -> DynamicColor {
        color.flags.set_named_color_space();
        color
    }

    if let Some(stripped) = s.strip_prefix('#') {
        let (ix, channels) = get_4bit_hex_channels(stripped)?;
        let color = color_from_4bit_hex(channels);
        // Hex colors are seen as if they are generated from the named `rgb()` color space
        // function.
        let mut color = DynamicColor::from_alpha_color(color);
        color.flags.set_named_color_space();
        return Ok((ix + 1, color));
    }
    let mut parser = Parser::new(s);
    if let Some(id) = parser.ident() {
        let mut buf = [0; LOWERCASE_BUF_SIZE];
        let id_lc = make_lowercase(id, &mut buf);
        let color = match id_lc {
            "rgb" | "rgba" => parser.rgb().map(set_from_named_color_space),
            "lab" => parser
                .lab(100.0, 1.25, ColorSpaceTag::Lab)
                .map(set_from_named_color_space),
            "lch" => parser
                .lch(100.0, 1.25, ColorSpaceTag::Lch)
                .map(set_from_named_color_space),
            "oklab" => parser
                .lab(1.0, 0.004, ColorSpaceTag::Oklab)
                .map(set_from_named_color_space),
            "oklch" => parser
                .lch(1.0, 0.004, ColorSpaceTag::Oklch)
                .map(set_from_named_color_space),
            "hsl" | "hsla" => parser.hsl().map(set_from_named_color_space),
            "hwb" => parser.hwb().map(set_from_named_color_space),
            "color" => parser.color(),
            _ => {
                if let Some(ix) = crate::x11_colors::lookup_palette_index(id_lc) {
                    let [r, g, b, a] = crate::x11_colors::COLORS[ix];
                    let mut color =
                        DynamicColor::from_alpha_color(AlphaColor::from_rgba8(r, g, b, a));
                    color.flags.set_named_color(ix);
                    Ok(color)
                } else {
                    Err(ParseError::UnknownColorIdentifier)
                }
            }
        }?;

        Ok((parser.ix, color))
    } else {
        Err(ParseError::UnknownColorSyntax)
    }
}

/// Parse a color string in CSS syntax into a color.
///
/// This parses the entire string; trailing characters cause an
/// [`ExpectedEndOfString`](ParseError::ExpectedEndOfString) parse error. Leading and trailing
/// whitespace are ignored. See also [`parse_color_prefix`].
///
/// # Errors
///
/// Tries to return a suitable error for any invalid string, but may be
/// a little lax on some details.
pub fn parse_color(s: &str) -> Result<DynamicColor, ParseError> {
    let s = s.trim();
    let (ix, color) = parse_color_prefix(s)?;

    if ix == s.len() {
        Ok(color)
    } else {
        Err(ParseError::ExpectedEndOfString)
    }
}

impl FromStr for DynamicColor {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
    }
}

impl<CS: ColorSpace> FromStr for AlphaColor<CS> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s).map(DynamicColor::to_alpha_color)
    }
}

impl<CS: ColorSpace> FromStr for OpaqueColor<CS> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
            .map(DynamicColor::to_alpha_color)
            .map(AlphaColor::discard_alpha)
    }
}

impl<CS: ColorSpace> FromStr for PremulColor<CS> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
            .map(DynamicColor::to_alpha_color)
            .map(AlphaColor::premultiply)
    }
}

/// Parse 4-bit color channels from a hex-encoded string.
///
/// Returns the parsed channels and the byte offset to the remainder of the string (i.e., the
/// number of hex characters parsed).
const fn get_4bit_hex_channels(hex_str: &str) -> Result<(usize, [u8; 8]), ParseError> {
    let mut hex = [0; 8];

    let mut i = 0;
    while i < 8 && i < hex_str.len() {
        if let Ok(h) = hex_from_ascii_byte(hex_str.as_bytes()[i]) {
            hex[i] = h;
            i += 1;
        } else {
            break;
        }
    }

    let four_bit_channels = match i {
        3 => [hex[0], hex[0], hex[1], hex[1], hex[2], hex[2], 15, 15],
        4 => [
            hex[0], hex[0], hex[1], hex[1], hex[2], hex[2], hex[3], hex[3],
        ],
        6 => [hex[0], hex[1], hex[2], hex[3], hex[4], hex[5], 15, 15],
        8 => hex,
        _ => return Err(ParseError::WrongNumberOfHexDigits),
    };

    Ok((i, four_bit_channels))
}

const fn hex_from_ascii_byte(b: u8) -> Result<u8, ()> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(()),
    }
}

const fn color_from_4bit_hex(components: [u8; 8]) -> AlphaColor<Srgb> {
    let [r0, r1, g0, g1, b0, b1, a0, a1] = components;
    AlphaColor::from_rgba8(
        (r0 << 4) | r1,
        (g0 << 4) | g1,
        (b0 << 4) | b1,
        (a0 << 4) | a1,
    )
}

impl FromStr for ColorSpaceTag {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; LOWERCASE_BUF_SIZE];
        match make_lowercase(s, &mut buf) {
            "srgb" => Ok(Self::Srgb),
            "srgb-linear" => Ok(Self::LinearSrgb),
            "lab" => Ok(Self::Lab),
            "lch" => Ok(Self::Lch),
            "oklab" => Ok(Self::Oklab),
            "oklch" => Ok(Self::Oklch),
            "display-p3" => Ok(Self::DisplayP3),
            "a98-rgb" => Ok(Self::A98Rgb),
            "prophoto-rgb" => Ok(Self::ProphotoRgb),
            "xyz-d50" => Ok(Self::XyzD50),
            "xyz" | "xyz-d65" => Ok(Self::XyzD65),
            _ => Err(ParseError::UnknownColorSpace),
        }
    }
}

const LOWERCASE_BUF_SIZE: usize = 32;

/// If the string contains any uppercase characters, make a lowercase copy
/// in the provided buffer space.
///
/// If anything goes wrong (including the buffer size being exceeded), return
/// the original string.
fn make_lowercase<'a>(s: &'a str, buf: &'a mut [u8; LOWERCASE_BUF_SIZE]) -> &'a str {
    let len = s.len();
    if len <= LOWERCASE_BUF_SIZE && s.as_bytes().iter().any(|c| c.is_ascii_uppercase()) {
        buf[..len].copy_from_slice(s.as_bytes());
        if let Ok(s_copy) = str::from_utf8_mut(&mut buf[..len]) {
            s_copy.make_ascii_lowercase();
            s_copy
        } else {
            s
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use crate::DynamicColor;

    use super::{parse_color, parse_color_prefix, Mode, ParseError, Parser};

    fn assert_close_color(c1: DynamicColor, c2: DynamicColor) {
        const EPSILON: f32 = 1e-4;
        assert_eq!(c1.cs, c2.cs);
        for i in 0..4 {
            assert!((c1.components[i] - c2.components[i]).abs() < EPSILON);
        }
    }

    fn assert_err(c: &str, err: ParseError) {
        assert_eq!(parse_color(c).unwrap_err(), err);
    }

    #[test]
    fn x11_color_names() {
        let red = parse_color("red").unwrap();
        assert_close_color(red, parse_color("rgb(255, 0, 0)").unwrap());
        assert_close_color(red, parse_color("\n rgb(255, 0, 0)\t ").unwrap());
        let lgy = parse_color("lightgoldenrodyellow").unwrap();
        assert_close_color(lgy, parse_color("rgb(250, 250, 210)").unwrap());
        let transparent = parse_color("transparent").unwrap();
        assert_close_color(transparent, parse_color("rgba(0, 0, 0, 0)").unwrap());
    }

    #[test]
    fn hex() {
        let red = parse_color("red").unwrap();
        assert_close_color(red, parse_color("#f00").unwrap());
        assert_close_color(red, parse_color("#f00f").unwrap());
        assert_close_color(red, parse_color("#ff0000ff").unwrap());
        assert_eq!(
            parse_color("#f00fa").unwrap_err(),
            ParseError::WrongNumberOfHexDigits
        );
    }

    #[test]
    fn consume_string() {
        assert_eq!(
            parse_color("#ff0000ffa").unwrap_err(),
            ParseError::ExpectedEndOfString
        );
        assert_eq!(
            parse_color("rgba(255, 100, 0, 1)a").unwrap_err(),
            ParseError::ExpectedEndOfString
        );
    }

    #[test]
    fn prefix() {
        for (color, trailing) in [
            ("color(rec2020 0.2 0.3 0.4 / 0.85)trailing", "trailing"),
            ("color(rec2020 0.2 0.3 0.4 / 0.85) ", " "),
            ("color(rec2020 0.2 0.3 0.4 / 0.85)", ""),
            ("red\0", "\0"),
            ("#ffftrailing", "trailing"),
            ("#fffffftr", "tr"),
        ] {
            assert_eq!(&color[parse_color_prefix(color).unwrap().0..], trailing);
        }
    }

    #[test]
    fn consume_comments() {
        for (s, remaining) in [
            ("/* abc */ def", " def"),
            ("/* *//* */abc", "abc"),
            ("/* /* */abc", "abc"),
        ] {
            let mut parser = Parser::new(s);
            assert!(parser.consume_comments().is_ok());
            assert_eq!(&parser.s[parser.ix..], remaining);
        }
    }

    #[test]
    fn alpha() {
        for (alpha, expected, mode) in [
            (", 10%", Ok(Some(0.1)), Mode::Legacy),
            ("/ 0.25", Ok(Some(0.25)), Mode::Modern),
            ("/ -0.3", Ok(Some(0.)), Mode::Modern),
            ("/ 110%", Ok(Some(1.)), Mode::Modern),
            ("", Ok(Some(1.)), Mode::Legacy),
            ("/ none", Ok(None), Mode::Modern),
        ] {
            let mut parser = Parser::new(alpha);
            let result = parser.alpha(mode);
            assert_eq!(result, expected,
                "Failed parsing specified alpha `{alpha}`. Expected: `{expected:?}`. Got: `{result:?}`.");
        }
    }

    #[test]
    fn angles() {
        for (angle, expected) in [
            ("90deg", 90.),
            ("1.5707963rad", 90.),
            ("100grad", 90.),
            ("0.25turn", 90.),
        ] {
            let mut parser = Parser::new(angle);
            let result = parser.angle().unwrap().unwrap();
            assert!((result - expected).abs() < 1e-4,
                    "Failed parsing specified angle `{angle}`. Expected: `{expected:?}`. Got: `{result:?}`.");
        }

        {
            let mut parser = Parser::new("none");
            assert_eq!(parser.angle().unwrap(), None);
        }

        assert_err(
            "hwb(1turns 20% 30% / 50%)",
            ParseError::UnknownAngleDimension,
        );
    }

    #[test]
    fn case_insensitive() {
        for (c1, c2) in [
            ("red", "ReD"),
            ("lightgoldenrodyellow", "LightGoldenRodYellow"),
            ("rgb(102, 51, 153)", "RGB(102, 51, 153)"),
            (
                "color(rec2020 0.2 0.3 0.4 / 0.85)",
                "CoLoR(ReC2020 0.2 0.3 0.4 / 0.85)",
            ),
            ("hwb(120deg 30% 50%)", "HwB(120DeG 30% 50%)"),
            ("hsl(none none none)", "HSL(NONE NONE NONE)"),
        ] {
            assert_close_color(parse_color(c1).unwrap(), parse_color(c2).unwrap());
        }
    }
}
