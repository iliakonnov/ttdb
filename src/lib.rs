#![feature(negative_impls)]
#![feature(specialization)]
#![feature(const_type_name)]
#![feature(optin_builtin_traits)]
#![feature(never_type)]
#![feature(const_generics)]
#![feature(maybe_uninit_extra)]
#![feature(new_uninit)]
#![allow(dead_code)]
#![allow(incomplete_features)]
pub mod hlist;
mod versions;
mod reservoir;
mod path;
mod api;
