# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.6](https://github.com/pacman82/double-trait/compare/v0.2.5...v0.2.6) - 2025-08-25

### Other

- migrated integration tests from double to dummies macro
- Introduce macro `dummies`
- extract double macro expansion into submodule of derive crate
- migrate tests from double to dummies macro
- Module dummy_impl is top level (again)
- Reuse double_trait for dummies macro
- Unit test for dummie on empty trait

## [0.2.5](https://github.com/pacman82/double-trait/compare/v0.2.4...v0.2.5) - 2025-08-19

### Added

- Support impl Iterator
- [**breaking**] Better error message in case of unsupported impl return type in

### Other

- *(deps)* bump tokio from 1.45.1 to 1.47.1
- *(deps)* bump async-trait from 0.1.88 to 0.1.89
- Unit test for compile_error
- *(deps)* bump proc-macro2 from 1.0.95 to 1.0.97
- *(deps)* bump syn from 2.0.103 to 2.0.105
- Doc comment for `trait_impl`

## [0.2.4](https://github.com/pacman82/double-trait/compare/v0.2.3...v0.2.4) - 2025-06-15

### Added

- Functions without return get a default empty implementation, rather than unimplemented.

### Other

- *(deps)* bump syn from 2.0.101 to 2.0.103

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