# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/pacman82/double-trait/compare/v0.2.2...v0.2.3) - 2025-06-09

### Added

- Default implementations of methods in double trait now mention method and trait name in unimplemented macro call

## [0.2.2](https://github.com/pacman82/double-trait/compare/v0.2.1...v0.2.2) - 2025-06-06

### Added

- Forwarding of associated types in trait

### Fixed

- default implementation for types

### Other

- improved comments
- Extract dummy_impl into own module
- Remove idea of transforming type items for double trait
- Respect existing defaults for associated types

## [0.2.1](https://github.com/pacman82/double-trait/compare/v0.2.0...v0.2.1) - 2025-06-05

### Added

- Derive std traits for Dummy

### Other

- test with async_trait
- fix dependencies to compile docstring
- More examples in doc string

## [0.2.0](https://github.com/pacman82/double-derive/compare/v0.1.3...v0.2.0) - 2025-06-04

### Breaking

- Split crate `double-derive` into a main crate `double-trait` and a supporting proc-macro crate `double-derive`.
- Introduce `Dummy` type implementing all doubles.

## [0.1.4](https://github.com/pacman82/double-derive/compare/v0.1.3...v0.1.4) - 2025-06-03

### Fixed

- Capitalize README.md in order for it to be picked up by cargo in the metainformation

## [0.1.3](https://github.com/pacman82/double-derive/compare/v0.1.2...v0.1.3) - 2025-06-03

### Added

- Avoid warnings from default implementations with named args.

## [0.1.2](https://github.com/pacman82/double-derive/compare/v0.1.1...v0.1.2) - 2025-06-03

### Added

- Improved compiler errors
- Rudimentary support for impl Future

### Other

- Extract is_impl_future into function

## [0.1.1](https://github.com/pacman82/double-derive/compare/v0.1.0...v0.1.1) - 2025-06-02

### Added

- Forwarding of associated methods
- Can forward async methods.
# Changelog