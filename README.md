[![crates.io](https://img.shields.io/crates/v/mocktoffel.svg)](https://crates.io/crates/mocktoffel)
# Mocktoffel
A library to easily mock your types without writing any boilerplate

# Installation

This crate can be added to your project by running
```
cargo add moctoffel
```

If this is a binary or a library that doesn't expose the mocks on its types, it is recommended to add it as a dev dependency by running:
```
cargo add moctoffel --dev
```

# Usage

The crate provides macros for mocking various constructs and will generate a new type for each mock. This ensures that the mock doesn’t intefere with other test code that requires the original type (such as testing the type in question) and yet is available for all tests.

The toffel macro is the initiator that will replace all the mock files with their respective macro types.

The mock macro will generate mocks for the structs or enums it is defined on. This will create a new type that mirrors the type.

The mocks generated are not test-gated at the moment. This may change in a future release. It is expected that the user wraps it in the test feature gate.

```
use mocktoffel::{toffel, mock};

[toffel]
pub struct Foo {
    #[mocked]
    bar: Bar,
    baz: String
}

#[mock]
pub struct Bar {
    some: String,
    #[mocked_with(Ok(1))]
    thing: Result<i32, ()>
}
```

# Scope
|Feature/Macro   | Toffel  | Mock  |
|---|---|---|
|Struct   |✓|✓|
|Enum   |✓|✓|
|Newtype Struct |✓|✓|
|Generics | - | |
|Associated Types|-||
|Macros   | | |
|Functions| | |
|Constants   | -| |

# MSRV

This crate uses unstable nightly features. These will be removed as features are stabilised, or replaced with stabilised polyfills with the unstable ones available as a nightly feature. The MSRV will be decided at that point. 

# Contributing

If you want to contribute a bug fix or a new feature, you can send a pull request. It is recommended to create a new issue for any major feature addition. The crate is dual-licensed under MIT & Apache license (ver 2.0). This repository follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

