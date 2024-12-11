// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Interpolation example
//!
//! Outputs a test page to stdout.
//!
//! Typical usage:
//!
//! ```sh
//! cargo run --example interp 'oklab(0.5 0.2 0)' 'rgb(0, 200, 0, 0.8)' oklab
//! ```

use color::{ColorSpaceTag, HueDirection};

fn main() {
    let mut args = std::env::args().skip(1);
    let c1_s = args.next().expect("give color as arg");
    let c1 = color::parse_color(&c1_s).expect("error parsing color 1");
    let c2_s = args.next().expect("give 2 colors as arg");
    let c2 = color::parse_color(&c2_s).expect("error parsing color 2");
    let cs_s_raw = args.next();
    let cs_s = cs_s_raw.as_deref().unwrap_or("srgb");
    let cs: ColorSpaceTag = cs_s.parse().expect("error parsing color space");
    const N: usize = 20;
    println!("<!DOCTYPE html>");
    println!("<html>");
    println!("<head>");
    println!("<style>");
    println!("div.g {{ height: 100px }}");
    let pct = 100.0 / N as f64;
    println!("span {{ width: {pct}%; display: inline-block; height: 100px; margin: 0 }}");
    let interpolator = c1.interpolate(c2, cs, HueDirection::default());
    for i in 0..=N {
        let t = i as f32 / (N as f32);
        let c = interpolator.eval(t);
        if i == 0 || i == N {
            println!("#s{i} {{ background: {c}; width: {}% }}", pct / 2.0);
        } else {
            println!("#s{i} {{ background: {c} }}");
        }
    }
    println!("#basic {{ background: linear-gradient(to right in {cs_s}, {c1_s}, {c2_s}) }}");
    println!("</style>");
    println!("</head>");
    println!("<body>");
    println!("<div>{c1_s} {c2_s} {cs_s}</div>");
    println!("<div class='g' id='basic'></div>");
    println!("<div class='g'>");
    for i in 0..=N {
        print!("<span id='s{i}'></span>");
    }
    println!();
    println!("</div>");
    println!("</body>");
    println!("</html>");
}
