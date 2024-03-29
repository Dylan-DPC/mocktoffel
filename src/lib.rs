#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(exact_size_is_empty)]
#![feature(extend_one)]
#![feature(extract_if)]
#![allow(unused)]
#![allow(clippy::module_name_repetitions)]
#![deny(rust_2018_idioms)]

//! This crate provides the capability of mocking local types easily without resorting to creating
//! the mock by hand or having to create mocks for each test.
//!
//! The crate provides macros for mocking various constructs and will generate a new type for each
//! mock. This ensures that the mock doesn't intefere with other test code that requires the
//! original type (such as testing the type in question) and yet is available for all tests.
//!
//! The [`macro@toffel`] macro is the initiator that will replace all the mock files with their
//! respective macro types.
//!
//! The [`macro@mock`] macro will generate mocks for the structs or enums it is defined on. This will
//! create a new type that mirrors the type.
//!
//! The mocks generated are not test-gated at the moment. This may change in a future release. It
//! is expected that the user wraps it in the `test` feature gate.
//!
//! ```
//! use mocktoffel::{toffel, mock};
//!
//! #[toffel]
//! pub struct Foo {
//!     #[mocked]
//!     bar: Bar,
//!     baz: String
//! }
//!
//! #[mock]
//! pub struct Bar {
//!     some: String,
//!     
//!     #[mocked_with(Ok(1))]
//!     thing: Result<i32, ()>
//! }
//!
//!     
//! ```
use crate::pimpl::MockContext;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Item, ItemImpl};
use toffel::Toffelise;

mod branch;
mod extract;
mod pimpl;
mod toffel;

/// The toffel macro is the initiator which will replace the fields with their corresponding mock
/// types. The fields need to be marked with the `#[mocked]` attribute. The fields without the
/// attribute will be retained as original.
///
/// The mocked fields need to have their mocks generated by using the [`macro@mock`] macro. Support
/// for other types that do not have the [`macro@mock`] macro or belong to other crates will be added
/// in a future release.
///
/// ```rust
/// use mocktoffel::{toffel, mock};
///
/// #[toffel]
/// struct Foo {
///   #[mocked]
///   bar: Bar,
///   qaz: String
/// }
///
/// #[mock]
/// struct Bar {}
/// ```
///  
#[proc_macro_attribute]
pub fn toffel(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as Item);

    match tokens {
        Item::Struct(s) => s.replace_mocks(),
        Item::Enum(e) => e.replace_mocks(),
        _ => todo!(),
    }
}

/// The `mock` macro will generate the corresponding mock for the struct or enum. In future this
/// will support mocking other types as well. The macro generates a new type named `<Name>Mock` that will match the
/// existing type with the same fields. The type is not test-gated at the moment, so the user is
/// expected to wrap the macro call in a feature gate.
///
/// The macro also creates a function on the type that crates a mocked object. This ensures that a
/// type can be created on types that do not implement `Default`. The method will use the custom
/// value of the field provided by using the `#[mocked_with(value)]` attribute. If this attribute
/// is not present, the default value of the type will be used.
///
/// For enums, the `#[mocked_with(value)]` needs to be placed on at most one variant as it will be
/// used as the default variant to construct the enum. The default value of the data should be
/// passed as a parameter to the attribute. If the attribute isn't specified, then the
/// default variant of the enum
///
/// To replace the original struct with the mocked struct in implementations and trait implements,
/// add the [`macro@mock_impl`] proc-macro to the `impl` and trait implementations where the type is
/// being used.  
///
/// ```rust
///
/// use mocktoffel::mock;
///
/// #[mock]
/// pub struct Bar {
///     some: String,
///     
///     #[mocked_with(Ok(1))]
///     thing: Result<i32, ()>
/// }
///
///     
/// ```
///

#[proc_macro_attribute]
pub fn mock(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = parse_macro_input!(input as Item);
    let mut mock = tokens.clone();
    let fields = match mock {
        Item::Struct(ref mut s) => {
            let name = format!("{}Mock", s.ident);
            s.ident = parse_str(name.as_str()).unwrap();
            extract::parse_fields_and_generate_for_values(s)
        }
        Item::Enum(ref mut e) => {
            let f = extract::parse_fields_and_generate_variant(e);
            let name = format!("{}Mock", e.ident);
            e.ident = parse_str(name.as_str()).unwrap();
            f
        }
        _ => todo!(),
    };

    extract::clean_out_attributes(&mut tokens);
    extract::clean_out_attributes(&mut mock);
    let mut original_and_mocked_struct = TokenStream::from(quote! {
        #tokens
        #mock
    });

    original_and_mocked_struct.extend_one(fields);
    original_and_mocked_struct
}

/// A helper macro that substitutes the original type with the mocked type on implementations and
/// trait implementations. The macro will take care of the occurrence of the type arguments and
/// return types.
///
/// ```rust
/// use mocktoffel::{mock, mock_impl};
/// #[mock]
/// pub struct Foo {
///     foo: String
/// }
///
/// #[mock_impl]
/// impl Foo {
///     pub fn bar(f: Foo) {}
/// }
/// ```
///
#[allow(clippy::redundant_clone)]
#[proc_macro_attribute]
pub fn mock_impl(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemImpl);
    let original_type = tokens.self_ty.clone();
    let context = MockContext::new(original_type);
    context.mock_impl(tokens)
}
