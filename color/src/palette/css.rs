// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The CSS named colors.
//!
//! The classic set of CSS named colors, also known as the [X11] colors.
//!
//! These are all of type [`AlphaColor<Srgb>`](AlphaColor).
//!
//! [X11]: https://en.wikipedia.org/wiki/X11_color_names

use crate::{AlphaColor, Srgb};

/// <div style="margin:2px 0"><span style="background-color:rgb(240, 248, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Alice blue (240, 248, 255, 255)</div>
pub const ALICE_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(240, 248, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(250, 235, 215);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Antique white (250, 235, 215, 255)</div>
pub const ANTIQUE_WHITE: AlphaColor<Srgb> = AlphaColor::from_rgb8(250, 235, 215);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 255, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Aqua (0, 255, 255, 255)</div>
pub const AQUA: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 255, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(127, 255, 212);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Aquamarine (127, 255, 212, 255)</div>
pub const AQUAMARINE: AlphaColor<Srgb> = AlphaColor::from_rgb8(127, 255, 212);
/// <div style="margin:2px 0"><span style="background-color:rgb(240, 255, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Azure (240, 255, 255, 255)</div>
pub const AZURE: AlphaColor<Srgb> = AlphaColor::from_rgb8(240, 255, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(245, 245, 220);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Beige (245, 245, 220, 255)</div>
pub const BEIGE: AlphaColor<Srgb> = AlphaColor::from_rgb8(245, 245, 220);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 228, 196);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Bisque (255, 228, 196, 255)</div>
pub const BISQUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 228, 196);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 0, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Black (0, 0, 0, 255)</div>
pub const BLACK: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 0, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 235, 205);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Blanched almond (255, 235, 205, 255)</div>
pub const BLANCHED_ALMOND: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 235, 205);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 0, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Blue (0, 0, 255, 255)</div>
pub const BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 0, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(138, 43, 226);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Blue violet (138, 43, 226, 255)</div>
pub const BLUE_VIOLET: AlphaColor<Srgb> = AlphaColor::from_rgb8(138, 43, 226);
/// <div style="margin:2px 0"><span style="background-color:rgb(165, 42, 42);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Brown (165, 42, 42, 255)</div>
pub const BROWN: AlphaColor<Srgb> = AlphaColor::from_rgb8(165, 42, 42);
/// <div style="margin:2px 0"><span style="background-color:rgb(222, 184, 135);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Burlywood (222, 184, 135, 255)</div>
pub const BURLYWOOD: AlphaColor<Srgb> = AlphaColor::from_rgb8(222, 184, 135);
/// <div style="margin:2px 0"><span style="background-color:rgb(95, 158, 160);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Cadet blue (95, 158, 160, 255)</div>
pub const CADET_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(95, 158, 160);
/// <div style="margin:2px 0"><span style="background-color:rgb(127, 255, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Chartreuse (127, 255, 0, 255)</div>
pub const CHARTREUSE: AlphaColor<Srgb> = AlphaColor::from_rgb8(127, 255, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(210, 105, 30);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Chocolate (210, 105, 30, 255)</div>
pub const CHOCOLATE: AlphaColor<Srgb> = AlphaColor::from_rgb8(210, 105, 30);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 127, 80);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Coral (255, 127, 80, 255)</div>
pub const CORAL: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 127, 80);
/// <div style="margin:2px 0"><span style="background-color:rgb(100, 149, 237);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Cornflower blue (100, 149, 237, 255)</div>
pub const CORNFLOWER_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(100, 149, 237);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 248, 220);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Cornsilk (255, 248, 220, 255)</div>
pub const CORNSILK: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 248, 220);
/// <div style="margin:2px 0"><span style="background-color:rgb(220, 20, 60);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Crimson (220, 20, 60, 255)</div>
pub const CRIMSON: AlphaColor<Srgb> = AlphaColor::from_rgb8(220, 20, 60);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 255, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Cyan (0, 255, 255, 255)</div>
pub const CYAN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 255, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 0, 139);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark blue (0, 0, 139, 255)</div>
pub const DARK_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 0, 139);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 139, 139);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark cyan (0, 139, 139, 255)</div>
pub const DARK_CYAN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 139, 139);
/// <div style="margin:2px 0"><span style="background-color:rgb(184, 134, 11);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark goldenrod (184, 134, 11, 255)</div>
pub const DARK_GOLDENROD: AlphaColor<Srgb> = AlphaColor::from_rgb8(184, 134, 11);
/// <div style="margin:2px 0"><span style="background-color:rgb(169, 169, 169);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark gray (169, 169, 169, 255)</div>
pub const DARK_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(169, 169, 169);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 100, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark green (0, 100, 0, 255)</div>
pub const DARK_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 100, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(189, 183, 107);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark khaki (189, 183, 107, 255)</div>
pub const DARK_KHAKI: AlphaColor<Srgb> = AlphaColor::from_rgb8(189, 183, 107);
/// <div style="margin:2px 0"><span style="background-color:rgb(139, 0, 139);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark magenta (139, 0, 139, 255)</div>
pub const DARK_MAGENTA: AlphaColor<Srgb> = AlphaColor::from_rgb8(139, 0, 139);
/// <div style="margin:2px 0"><span style="background-color:rgb(85, 107, 47);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark olive green (85, 107, 47, 255)</div>
pub const DARK_OLIVE_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(85, 107, 47);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 140, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark orange (255, 140, 0, 255)</div>
pub const DARK_ORANGE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 140, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(153, 50, 204);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark orchid (153, 50, 204, 255)</div>
pub const DARK_ORCHID: AlphaColor<Srgb> = AlphaColor::from_rgb8(153, 50, 204);
/// <div style="margin:2px 0"><span style="background-color:rgb(139, 0, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark red (139, 0, 0, 255)</div>
pub const DARK_RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(139, 0, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(233, 150, 122);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark salmon (233, 150, 122, 255)</div>
pub const DARK_SALMON: AlphaColor<Srgb> = AlphaColor::from_rgb8(233, 150, 122);
/// <div style="margin:2px 0"><span style="background-color:rgb(143, 188, 143);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark sea green (143, 188, 143, 255)</div>
pub const DARK_SEA_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(143, 188, 143);
/// <div style="margin:2px 0"><span style="background-color:rgb(72, 61, 139);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark slate blue (72, 61, 139, 255)</div>
pub const DARK_SLATE_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(72, 61, 139);
/// <div style="margin:2px 0"><span style="background-color:rgb(47, 79, 79);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark slate gray (47, 79, 79, 255)</div>
pub const DARK_SLATE_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(47, 79, 79);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 206, 209);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark turquoise (0, 206, 209, 255)</div>
pub const DARK_TURQUOISE: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 206, 209);
/// <div style="margin:2px 0"><span style="background-color:rgb(148, 0, 211);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dark violet (148, 0, 211, 255)</div>
pub const DARK_VIOLET: AlphaColor<Srgb> = AlphaColor::from_rgb8(148, 0, 211);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 20, 147);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Deep pink (255, 20, 147, 255)</div>
pub const DEEP_PINK: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 20, 147);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 191, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Deep sky blue (0, 191, 255, 255)</div>
pub const DEEP_SKY_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 191, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(105, 105, 105);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dim gray (105, 105, 105, 255)</div>
pub const DIM_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(105, 105, 105);
/// <div style="margin:2px 0"><span style="background-color:rgb(30, 144, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Dodger blue (30, 144, 255, 255)</div>
pub const DODGER_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(30, 144, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(178, 34, 34);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Firebrick (178, 34, 34, 255)</div>
pub const FIREBRICK: AlphaColor<Srgb> = AlphaColor::from_rgb8(178, 34, 34);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 250, 240);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Floral white (255, 250, 240, 255)</div>
pub const FLORAL_WHITE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 250, 240);
/// <div style="margin:2px 0"><span style="background-color:rgb(34, 139, 34);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Forest green (34, 139, 34, 255)</div>
pub const FOREST_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(34, 139, 34);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 0, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Fuchsia (255, 0, 255, 255)</div>
pub const FUCHSIA: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 0, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(220, 220, 220);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Gainsboro (220, 220, 220, 255)</div>
pub const GAINSBORO: AlphaColor<Srgb> = AlphaColor::from_rgb8(220, 220, 220);
/// <div style="margin:2px 0"><span style="background-color:rgb(248, 248, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Ghost white (248, 248, 255, 255)</div>
pub const GHOST_WHITE: AlphaColor<Srgb> = AlphaColor::from_rgb8(248, 248, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 215, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Gold (255, 215, 0, 255)</div>
pub const GOLD: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 215, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(218, 165, 32);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Goldenrod (218, 165, 32, 255)</div>
pub const GOLDENROD: AlphaColor<Srgb> = AlphaColor::from_rgb8(218, 165, 32);
/// <div style="margin:2px 0"><span style="background-color:rgb(128, 128, 128);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Gray (128, 128, 128, 255)</div>
pub const GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(128, 128, 128);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 128, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Green (0, 128, 0, 255)</div>
pub const GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 128, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(173, 255, 47);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Green yellow (173, 255, 47, 255)</div>
pub const GREEN_YELLOW: AlphaColor<Srgb> = AlphaColor::from_rgb8(173, 255, 47);
/// <div style="margin:2px 0"><span style="background-color:rgb(240, 255, 240);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Honeydew (240, 255, 240, 255)</div>
pub const HONEYDEW: AlphaColor<Srgb> = AlphaColor::from_rgb8(240, 255, 240);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 105, 180);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Hot pink (255, 105, 180, 255)</div>
pub const HOT_PINK: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 105, 180);
/// <div style="margin:2px 0"><span style="background-color:rgb(205, 92, 92);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Indian red (205, 92, 92, 255)</div>
pub const INDIAN_RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(205, 92, 92);
/// <div style="margin:2px 0"><span style="background-color:rgb(75, 0, 130);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Indigo (75, 0, 130, 255)</div>
pub const INDIGO: AlphaColor<Srgb> = AlphaColor::from_rgb8(75, 0, 130);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 255, 240);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Ivory (255, 255, 240, 255)</div>
pub const IVORY: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 255, 240);
/// <div style="margin:2px 0"><span style="background-color:rgb(240, 230, 140);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Khaki (240, 230, 140, 255)</div>
pub const KHAKI: AlphaColor<Srgb> = AlphaColor::from_rgb8(240, 230, 140);
/// <div style="margin:2px 0"><span style="background-color:rgb(230, 230, 250);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lavender (230, 230, 250, 255)</div>
pub const LAVENDER: AlphaColor<Srgb> = AlphaColor::from_rgb8(230, 230, 250);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 240, 245);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lavender blush (255, 240, 245, 255)</div>
pub const LAVENDER_BLUSH: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 240, 245);
/// <div style="margin:2px 0"><span style="background-color:rgb(124, 252, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lawn green (124, 252, 0, 255)</div>
pub const LAWN_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(124, 252, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 250, 205);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lemon chiffon (255, 250, 205, 255)</div>
pub const LEMON_CHIFFON: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 250, 205);
/// <div style="margin:2px 0"><span style="background-color:rgb(173, 216, 230);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light blue (173, 216, 230, 255)</div>
pub const LIGHT_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(173, 216, 230);
/// <div style="margin:2px 0"><span style="background-color:rgb(240, 128, 128);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light coral (240, 128, 128, 255)</div>
pub const LIGHT_CORAL: AlphaColor<Srgb> = AlphaColor::from_rgb8(240, 128, 128);
/// <div style="margin:2px 0"><span style="background-color:rgb(224, 255, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light cyan (224, 255, 255, 255)</div>
pub const LIGHT_CYAN: AlphaColor<Srgb> = AlphaColor::from_rgb8(224, 255, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(250, 250, 210);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light goldenrod yellow (250, 250, 210, 255)</div>
pub const LIGHT_GOLDENROD_YELLOW: AlphaColor<Srgb> = AlphaColor::from_rgb8(250, 250, 210);
/// <div style="margin:2px 0"><span style="background-color:rgb(211, 211, 211);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light gray (211, 211, 211, 255)</div>
pub const LIGHT_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(211, 211, 211);
/// <div style="margin:2px 0"><span style="background-color:rgb(144, 238, 144);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light green (144, 238, 144, 255)</div>
pub const LIGHT_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(144, 238, 144);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 182, 193);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light pink (255, 182, 193, 255)</div>
pub const LIGHT_PINK: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 182, 193);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 160, 122);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light salmon (255, 160, 122, 255)</div>
pub const LIGHT_SALMON: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 160, 122);
/// <div style="margin:2px 0"><span style="background-color:rgb(32, 178, 170);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light sea green (32, 178, 170, 255)</div>
pub const LIGHT_SEA_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(32, 178, 170);
/// <div style="margin:2px 0"><span style="background-color:rgb(135, 206, 250);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light sky blue (135, 206, 250, 255)</div>
pub const LIGHT_SKY_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(135, 206, 250);
/// <div style="margin:2px 0"><span style="background-color:rgb(119, 136, 153);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light slate gray (119, 136, 153, 255)</div>
pub const LIGHT_SLATE_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(119, 136, 153);
/// <div style="margin:2px 0"><span style="background-color:rgb(176, 196, 222);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light steel blue (176, 196, 222, 255)</div>
pub const LIGHT_STEEL_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(176, 196, 222);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 255, 224);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Light yellow (255, 255, 224, 255)</div>
pub const LIGHT_YELLOW: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 255, 224);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 255, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lime (0, 255, 0, 255)</div>
pub const LIME: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 255, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(50, 205, 50);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Lime green (50, 205, 50, 255)</div>
pub const LIME_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(50, 205, 50);
/// <div style="margin:2px 0"><span style="background-color:rgb(250, 240, 230);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Linen (250, 240, 230, 255)</div>
pub const LINEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(250, 240, 230);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 0, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Magenta (255, 0, 255, 255)</div>
pub const MAGENTA: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 0, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(128, 0, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Maroon (128, 0, 0, 255)</div>
pub const MAROON: AlphaColor<Srgb> = AlphaColor::from_rgb8(128, 0, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(102, 205, 170);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium aquamarine (102, 205, 170, 255)</div>
pub const MEDIUM_AQUAMARINE: AlphaColor<Srgb> = AlphaColor::from_rgb8(102, 205, 170);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 0, 205);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium blue (0, 0, 205, 255)</div>
pub const MEDIUM_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 0, 205);
/// <div style="margin:2px 0"><span style="background-color:rgb(186, 85, 211);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium orchid (186, 85, 211, 255)</div>
pub const MEDIUM_ORCHID: AlphaColor<Srgb> = AlphaColor::from_rgb8(186, 85, 211);
/// <div style="margin:2px 0"><span style="background-color:rgb(147, 112, 219);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium purple (147, 112, 219, 255)</div>
pub const MEDIUM_PURPLE: AlphaColor<Srgb> = AlphaColor::from_rgb8(147, 112, 219);
/// <div style="margin:2px 0"><span style="background-color:rgb(60, 179, 113);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium sea green (60, 179, 113, 255)</div>
pub const MEDIUM_SEA_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(60, 179, 113);
/// <div style="margin:2px 0"><span style="background-color:rgb(123, 104, 238);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium slate blue (123, 104, 238, 255)</div>
pub const MEDIUM_SLATE_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(123, 104, 238);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 250, 154);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium spring green (0, 250, 154, 255)</div>
pub const MEDIUM_SPRING_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 250, 154);
/// <div style="margin:2px 0"><span style="background-color:rgb(72, 209, 204);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium turquoise (72, 209, 204, 255)</div>
pub const MEDIUM_TURQUOISE: AlphaColor<Srgb> = AlphaColor::from_rgb8(72, 209, 204);
/// <div style="margin:2px 0"><span style="background-color:rgb(199, 21, 133);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Medium violet red (199, 21, 133, 255)</div>
pub const MEDIUM_VIOLET_RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(199, 21, 133);
/// <div style="margin:2px 0"><span style="background-color:rgb(25, 25, 112);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Midnight blue (25, 25, 112, 255)</div>
pub const MIDNIGHT_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(25, 25, 112);
/// <div style="margin:2px 0"><span style="background-color:rgb(245, 255, 250);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Mint cream (245, 255, 250, 255)</div>
pub const MINT_CREAM: AlphaColor<Srgb> = AlphaColor::from_rgb8(245, 255, 250);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 228, 225);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Misty rose (255, 228, 225, 255)</div>
pub const MISTY_ROSE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 228, 225);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 228, 181);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Moccasin (255, 228, 181, 255)</div>
pub const MOCCASIN: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 228, 181);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 222, 173);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Navajo white (255, 222, 173, 255)</div>
pub const NAVAJO_WHITE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 222, 173);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 0, 128);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Navy (0, 0, 128, 255)</div>
pub const NAVY: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 0, 128);
/// <div style="margin:2px 0"><span style="background-color:rgb(253, 245, 230);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Old lace (253, 245, 230, 255)</div>
pub const OLD_LACE: AlphaColor<Srgb> = AlphaColor::from_rgb8(253, 245, 230);
/// <div style="margin:2px 0"><span style="background-color:rgb(128, 128, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Olive (128, 128, 0, 255)</div>
pub const OLIVE: AlphaColor<Srgb> = AlphaColor::from_rgb8(128, 128, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(107, 142, 35);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Olive drab (107, 142, 35, 255)</div>
pub const OLIVE_DRAB: AlphaColor<Srgb> = AlphaColor::from_rgb8(107, 142, 35);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 165, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Orange (255, 165, 0, 255)</div>
pub const ORANGE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 165, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 69, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Orange red (255, 69, 0, 255)</div>
pub const ORANGE_RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 69, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(218, 112, 214);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Orchid (218, 112, 214, 255)</div>
pub const ORCHID: AlphaColor<Srgb> = AlphaColor::from_rgb8(218, 112, 214);
/// <div style="margin:2px 0"><span style="background-color:rgb(238, 232, 170);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Pale goldenrod (238, 232, 170, 255)</div>
pub const PALE_GOLDENROD: AlphaColor<Srgb> = AlphaColor::from_rgb8(238, 232, 170);
/// <div style="margin:2px 0"><span style="background-color:rgb(152, 251, 152);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Pale green (152, 251, 152, 255)</div>
pub const PALE_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(152, 251, 152);
/// <div style="margin:2px 0"><span style="background-color:rgb(175, 238, 238);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Pale turquoise (175, 238, 238, 255)</div>
pub const PALE_TURQUOISE: AlphaColor<Srgb> = AlphaColor::from_rgb8(175, 238, 238);
/// <div style="margin:2px 0"><span style="background-color:rgb(219, 112, 147);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Pale violet red (219, 112, 147, 255)</div>
pub const PALE_VIOLET_RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(219, 112, 147);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 239, 213);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Papaya whip (255, 239, 213, 255)</div>
pub const PAPAYA_WHIP: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 239, 213);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 218, 185);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Peach puff (255, 218, 185, 255)</div>
pub const PEACH_PUFF: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 218, 185);
/// <div style="margin:2px 0"><span style="background-color:rgb(205, 133, 63);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Peru (205, 133, 63, 255)</div>
pub const PERU: AlphaColor<Srgb> = AlphaColor::from_rgb8(205, 133, 63);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 192, 203);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Pink (255, 192, 203, 255)</div>
pub const PINK: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 192, 203);
/// <div style="margin:2px 0"><span style="background-color:rgb(221, 160, 221);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Plum (221, 160, 221, 255)</div>
pub const PLUM: AlphaColor<Srgb> = AlphaColor::from_rgb8(221, 160, 221);
/// <div style="margin:2px 0"><span style="background-color:rgb(176, 224, 230);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Powder blue (176, 224, 230, 255)</div>
pub const POWDER_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(176, 224, 230);
/// <div style="margin:2px 0"><span style="background-color:rgb(128, 0, 128);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Purple (128, 0, 128, 255)</div>
pub const PURPLE: AlphaColor<Srgb> = AlphaColor::from_rgb8(128, 0, 128);
/// <div style="margin:2px 0"><span style="background-color:rgb(102, 51, 153);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Rebecca purple (102, 51, 153, 255)</div>
pub const REBECCA_PURPLE: AlphaColor<Srgb> = AlphaColor::from_rgb8(102, 51, 153);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 0, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Red (255, 0, 0, 255)</div>
pub const RED: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 0, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(188, 143, 143);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Rosy brown (188, 143, 143, 255)</div>
pub const ROSY_BROWN: AlphaColor<Srgb> = AlphaColor::from_rgb8(188, 143, 143);
/// <div style="margin:2px 0"><span style="background-color:rgb(65, 105, 225);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Royal blue (65, 105, 225, 255)</div>
pub const ROYAL_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(65, 105, 225);
/// <div style="margin:2px 0"><span style="background-color:rgb(139, 69, 19);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Saddle brown (139, 69, 19, 255)</div>
pub const SADDLE_BROWN: AlphaColor<Srgb> = AlphaColor::from_rgb8(139, 69, 19);
/// <div style="margin:2px 0"><span style="background-color:rgb(250, 128, 114);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Salmon (250, 128, 114, 255)</div>
pub const SALMON: AlphaColor<Srgb> = AlphaColor::from_rgb8(250, 128, 114);
/// <div style="margin:2px 0"><span style="background-color:rgb(244, 164, 96);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Sandy brown (244, 164, 96, 255)</div>
pub const SANDY_BROWN: AlphaColor<Srgb> = AlphaColor::from_rgb8(244, 164, 96);
/// <div style="margin:2px 0"><span style="background-color:rgb(46, 139, 87);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Sea green (46, 139, 87, 255)</div>
pub const SEA_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(46, 139, 87);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 245, 238);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Seashell (255, 245, 238, 255)</div>
pub const SEASHELL: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 245, 238);
/// <div style="margin:2px 0"><span style="background-color:rgb(160, 82, 45);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Sienna (160, 82, 45, 255)</div>
pub const SIENNA: AlphaColor<Srgb> = AlphaColor::from_rgb8(160, 82, 45);
/// <div style="margin:2px 0"><span style="background-color:rgb(192, 192, 192);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Silver (192, 192, 192, 255)</div>
pub const SILVER: AlphaColor<Srgb> = AlphaColor::from_rgb8(192, 192, 192);
/// <div style="margin:2px 0"><span style="background-color:rgb(135, 206, 235);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Sky blue (135, 206, 235, 255)</div>
pub const SKY_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(135, 206, 235);
/// <div style="margin:2px 0"><span style="background-color:rgb(106, 90, 205);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Slate blue (106, 90, 205, 255)</div>
pub const SLATE_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(106, 90, 205);
/// <div style="margin:2px 0"><span style="background-color:rgb(112, 128, 144);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Slate gray (112, 128, 144, 255)</div>
pub const SLATE_GRAY: AlphaColor<Srgb> = AlphaColor::from_rgb8(112, 128, 144);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 250, 250);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Snow (255, 250, 250, 255)</div>
pub const SNOW: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 250, 250);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 255, 127);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Spring green (0, 255, 127, 255)</div>
pub const SPRING_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 255, 127);
/// <div style="margin:2px 0"><span style="background-color:rgb(70, 130, 180);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Steel blue (70, 130, 180, 255)</div>
pub const STEEL_BLUE: AlphaColor<Srgb> = AlphaColor::from_rgb8(70, 130, 180);
/// <div style="margin:2px 0"><span style="background-color:rgb(210, 180, 140);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Tan (210, 180, 140, 255)</div>
pub const TAN: AlphaColor<Srgb> = AlphaColor::from_rgb8(210, 180, 140);
/// <div style="margin:2px 0"><span style="background-color:rgb(0, 128, 128);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Teal (0, 128, 128, 255)</div>
pub const TEAL: AlphaColor<Srgb> = AlphaColor::from_rgb8(0, 128, 128);
/// <div style="margin:2px 0"><span style="background-color:rgb(216, 191, 216);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Thistle (216, 191, 216, 255)</div>
pub const THISTLE: AlphaColor<Srgb> = AlphaColor::from_rgb8(216, 191, 216);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 99, 71);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Tomato (255, 99, 71, 255)</div>
pub const TOMATO: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 99, 71);
/// <div style="margin:2px 0"><span style="background-color:rgba(0, 0, 0, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Transparent (0, 0, 0, 0)</div>
pub const TRANSPARENT: AlphaColor<Srgb> = AlphaColor::from_rgba8(0, 0, 0, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(64, 224, 208);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Turquoise (64, 224, 208, 255)</div>
pub const TURQUOISE: AlphaColor<Srgb> = AlphaColor::from_rgb8(64, 224, 208);
/// <div style="margin:2px 0"><span style="background-color:rgb(238, 130, 238);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Violet (238, 130, 238, 255)</div>
pub const VIOLET: AlphaColor<Srgb> = AlphaColor::from_rgb8(238, 130, 238);
/// <div style="margin:2px 0"><span style="background-color:rgb(245, 222, 179);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Wheat (245, 222, 179, 255)</div>
pub const WHEAT: AlphaColor<Srgb> = AlphaColor::from_rgb8(245, 222, 179);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 255, 255);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>White (255, 255, 255, 255)</div>
pub const WHITE: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 255, 255);
/// <div style="margin:2px 0"><span style="background-color:rgb(245, 245, 245);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>White smoke (245, 245, 245, 255)</div>
pub const WHITE_SMOKE: AlphaColor<Srgb> = AlphaColor::from_rgb8(245, 245, 245);
/// <div style="margin:2px 0"><span style="background-color:rgb(255, 255, 0);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Yellow (255, 255, 0, 255)</div>
pub const YELLOW: AlphaColor<Srgb> = AlphaColor::from_rgb8(255, 255, 0);
/// <div style="margin:2px 0"><span style="background-color:rgb(154, 205, 50);padding:0 0.7em;margin-right:0.5em;border:1px solid"></span>Yellow green (154, 205, 50, 255)</div>
pub const YELLOW_GREEN: AlphaColor<Srgb> = AlphaColor::from_rgb8(154, 205, 50);
