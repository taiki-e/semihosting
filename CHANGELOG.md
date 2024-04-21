# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

- Respect `RUSTC_WRAPPER` in rustc version detection.

- Documentation improvements.

## [0.1.7] - 2024-03-21

- Support Xtensa (OpenOCD Semihosting) under the `openocd-semihosting` feature. ([#9](https://github.com/taiki-e/semihosting/pull/9))

## [0.1.6] - 2024-03-02

- Documentation improvements.

## [0.1.5] - 2023-12-27

- Expose raw syscall interface as public API. ([#7](https://github.com/taiki-e/semihosting/pull/7), thanks @t-moe)

## [0.1.4] - 2023-08-25

- Update `unwinding` to 0.2.

## [0.1.3] - 2023-07-27

- Fix build error on MIPS32r6 and MIPS64r6 since [nightly-2023-07-19's target_arch change](https://github.com/rust-lang/rust/pull/112374).

## [0.1.2] - 2023-05-06

- Enable `portable-atomic`'s `require-cas` feature to display helpful error messages to users on targets requiring additional action on the user side to provide atomic CAS.

## [0.1.1] - 2023-04-09

- Improve panic message on stable.

## [0.1.0] - 2023-03-22

Initial release

[Unreleased]: https://github.com/taiki-e/semihosting/compare/v0.1.7...HEAD
[0.1.7]: https://github.com/taiki-e/semihosting/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/taiki-e/semihosting/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/taiki-e/semihosting/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/taiki-e/semihosting/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/taiki-e/semihosting/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/taiki-e/semihosting/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/semihosting/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/semihosting/releases/tag/v0.1.0
