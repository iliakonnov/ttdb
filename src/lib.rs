#![feature(
    negative_impls,
    specialization,
    const_type_name,
    optin_builtin_traits,
    never_type,
    const_generics,
    maybe_uninit_extra,
    new_uninit,
    vec_into_raw_parts,
    const_raw_ptr_deref
)]
#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![cfg_attr(debug_assertions, allow(dead_code, missing_docs))]
#![allow(
    incomplete_features,
    clippy::wildcard_imports,
    clippy::type_repetition_in_bounds,
    clippy::use_self
)]
#[macro_use] pub mod hlist;
mod versions;
mod reservoir;
mod path;
mod api;
