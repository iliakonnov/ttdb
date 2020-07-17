#![feature(
    negative_impls,
    specialization,
    const_type_name,
    never_type,
    exhaustive_patterns,
    const_generics,
    maybe_uninit_extra,
    new_uninit,
    vec_into_raw_parts,
    const_raw_ptr_deref,
    generic_associated_types,
    trivial_bounds,
    associated_type_defaults,
    trait_alias
)]
#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
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
    clippy::use_self,
    clippy::missing_errors_doc
)]
#[macro_use] mod fntools;

#[macro_use] pub mod hlist;
#[macro_use] pub mod versions;
pub mod reservoir;
#[macro_use] pub mod path;

pub mod utils;
pub mod error;
pub mod api;
pub mod storage;
