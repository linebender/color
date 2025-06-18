<div align="center">

# Color

A library for representing and manipulating colors

[![Linebender Zulip, #color channel](https://img.shields.io/badge/Linebender-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)
[![dependency status](https://deps.rs/repo/github/linebender/color/status.svg)](https://deps.rs/repo/github/linebender/color)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)
[![Build status](https://github.com/linebender/color/workflows/CI/badge.svg)](https://github.com/linebender/color/actions)
[![Crates.io](https://img.shields.io/crates/v/color.svg)](https://crates.io/crates/color)
[![Docs](https://docs.rs/color/badge.svg)](https://docs.rs/color)

</div>

The Color library provides functionality for representing, converting, parsing, serializing, and manipulating colors in a variety of color spaces.
It closely follows the [CSS Color Level 4] draft spec.

## Minimum supported Rust Version (MSRV)

This version of Color has been verified to compile with **Rust 1.82** and later.

Future versions of Color might increase the Rust version requirement.
It will not be treated as a breaking change and as such can even happen with small patch releases.

<details>
<summary>Click here if compiling fails.</summary>

As time has passed, some of Color's dependencies could have released versions with a higher Rust requirement.
If you encounter a compilation issue due to a dependency and don't want to upgrade your Rust toolchain, then you could downgrade the dependency.

```sh
# Use the problematic dependency's name and version
cargo update -p package_name --precise 0.1.1
```
</details>

## Community

[![Linebender Zulip](https://img.shields.io/badge/Linebender%20Zulip-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)

Discussion of Color development happens in the [Linebender Zulip](https://xi.zulipchat.com/), specifically the [#color channel](https://xi.zulipchat.com/#narrow/channel/466849-color).
All public content can be read without logging in.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome by pull request. The [Rust code of conduct] applies.
Please feel free to add your name to the [AUTHORS] file in any substantive pull request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct
[AUTHORS]: ./AUTHORS
[CSS Color Level 4]: https://www.w3.org/TR/css-color-4/
