// To add new error:
// - Write bound
// - Add to Error enum
// - Add to impl_bounded macro definition
// - Add to kinds mod
// - Add to make_checkers macro call
// - Add to SetKind trait implementation
// - Extend generate_aliases macro call if needed
// FIXME: make single macro to do all these things

use crate::utils::{Is, IsNot};

// Defines bounds on generic arguments of Error enum.
mod bounds {
    pub trait NoSuchKey {}
    impl<N: super::Is<!>> NoSuchKey for N {}
    impl NoSuchKey for () {}

    pub trait StorageError {}
    impl<T> StorageError for T {}

    pub trait SerializationError {}
    impl<N: super::Is<!>> SerializationError for N {}
    impl SerializationError for Box<dyn std::error::Error> {}

    pub trait DeserializationError {}
    impl<N: super::Is<!>> DeserializationError for N {}
    impl DeserializationError for Box<dyn std::error::Error> {}
}

#[derive(Debug, Copy, Clone)]
#[allow(clippy::pub_enum_variant_names)]
pub enum Error<
    TNoSuchKey: bounds::NoSuchKey = !,
    TStorageError: bounds::StorageError = !,
    TSerializationError: bounds::SerializationError = !,
    TDeserializationError: bounds::DeserializationError = !,
> {
    NoSuchKey(TNoSuchKey),
    StorageError(TStorageError),
    SerializationError(TSerializationError),
    DeserializationError(TDeserializationError),
}

macro_rules! impl_bounded {
    {
        $(
            impl<#bounds$(, $generics:ident)*> [$trait:path] for #error $(where ($($where:tt)*))?
            { $($body:tt)* }
        )*
    } => {
        $(
            impl<
                TNoSuchKey: crate::error::bounds::NoSuchKey,
                TStorageError: crate::error::bounds::StorageError,
                TSerializationError: crate::error::bounds::SerializationError,
                TDeserializationError: crate::error::bounds::DeserializationError
                $(, $generics)*
            > $trait for crate::error::Error<
                TNoSuchKey,
                TStorageError,
                TSerializationError,
                TDeserializationError,
            > $(where $($where)*)? {
                $($body)*
            }
        )*
    }
}

pub mod kinds {
    use super::bounds;

    pub trait IsKind {
        type Inner;
    }

    macro_rules! define_kinds {
        (
            $(pub struct $name:ident<$ty:ident>;)*
        ) => {
            // Define structs
            $(
                #[derive(Debug, Clone, Copy)]
                pub struct $name<$ty = super::Placeholder>(pub $ty);

                impl<$ty: bounds::$name> IsKind for $name<$ty> {
                    type Inner = $ty;
                }

                impl_bounded! {
                    impl<#bounds> [From<$name<$ty>>] for #error {
                        fn from(x: $name<$ty>) -> Self {
                            super::Error::$name(x.0)
                        }
                    }
                }
            )*
        };
    }
    define_kinds! {
        pub struct NoSuchKey<TNoSuchKey>;
        pub struct StorageError<TStorageError>;
        pub struct SerializationError<TSerializationError>;
        pub struct DeserializationError<TDeserializationError>;
    }
}

// Defines helper traits to check is some error is possible (not never)
mod checks {
    use super::{Placeholder, bounds, kinds};
    use crate::utils::{Bool, No, TestIs};

    /// Implemented only for errors.
    /// Allows getting type of some kind from Error.
    pub trait GetTypeOf<Kind> {
        type Type;
    }

    impl_bounded! {
        impl<#bounds> [GetTypeOf<Placeholder>] for #error {
            type Type = Placeholder;
        }
    }

    macro_rules! make_checkers {
        ($(pub trait $checker:ident for $kind:ident<$ty:ident>;)*) => {
            // GetTypeOf
            impl_bounded! {
                $(
                    impl<#bounds, T> [GetTypeOf<kinds::$kind<T>>] for #error where (T: bounds::$kind) {
                        default type Type = $ty;
                    }
                    // Placeholder is good too
                    impl<#bounds> [GetTypeOf<kinds::$kind<Placeholder>>] for #error {
                        type Type = $ty;
                    }
                )*
            }

            // KindsEnabled
            pub trait KindsEnabled {
                $(type $kind: Bool = No;)*
            }
            impl_bounded! {
                impl<#bounds> [KindsEnabled] for #error {
                    $(
                        type $kind = <
                            <
                                <Self as GetTypeOf<kinds::$kind>>::Type
                            as TestIs<!>>::Res
                        as Bool>::Not;
                    )*
                }
            }
        };
    }

    make_checkers!(
        pub trait IsNoSuchKey for NoSuchKey<TNoSuchKey>;
        pub trait IsStorageError for StorageError<TStorageError>;
        pub trait IsSerializationError for SerializationError<TSerializationError>;
        pub trait IsDeserializationError for DeserializationError<TDeserializationError>;
    );
}
pub use checks::{GetTypeOf, KindsEnabled};
use kinds::IsKind;

// Now some utils

#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct Placeholder;

pub trait SetKind<Kind> {
    type Res;
}

impl_bounded! {
    impl<#bounds> [SetKind<Placeholder>] for #error {
        type Res = Self;
    }
    impl<#bounds, New> [SetKind<kinds::NoSuchKey<New>>] for #error where (New: bounds::NoSuchKey) {
        type Res = Error<New, TStorageError, TSerializationError, TDeserializationError>;
    }
    impl<#bounds, New> [SetKind<kinds::StorageError<New>>] for #error where (New: bounds::StorageError) {
        type Res = Error<TNoSuchKey, New, TSerializationError, TDeserializationError>;
    }
    impl<#bounds, New> [SetKind<kinds::SerializationError<New>>] for #error where (New: bounds::SerializationError) {
        type Res = Error<TNoSuchKey, TStorageError, New, TDeserializationError>;
    }
    impl<#bounds, New> [SetKind<kinds::DeserializationError<New>>] for #error where (New: bounds::DeserializationError) {
        type Res = Error<TNoSuchKey, TStorageError, TSerializationError, New>;
    }
}

// Expands to:
/*
type Extend<A=Placeholder, B=Placeholder, Base=Error> = <
        <Base as SetKind<A>>::Res
    as SetKind<B>>::Res;
*/
macro_rules! generate_alias {
    (
        pub type $extend:ident <$($t:ident)*> ($base:ident)
    ) => {
        pub type $extend<$($t = Placeholder, )* $base = Error> =
            generate_alias!(@reverse <$base> [] $($t)*)
        ;
    };
    (@reverse <$base:ident> [$first:ident $($rest:ident)*] $($reversed:ident)*) => {
        // Continue reversing
        generate_alias!(@reverse <$base> [$($rest)*] $first $($reversed)*)
    };
    (@reverse <$base:ident> [] $($reversed:ident)*) => {
        // Reversed. Now go to @type
        generate_alias!(@type <$base> $($reversed)*)  // base case
    };
    (@type <$base:ident> $curr:ident $($t:ident)*) => {
        <
            generate_alias!(@type <$base> $($t)*)
            as SetKind<$curr>
        >::Res
    };
    (@type <$base:ident>) => {
        // Done with @type
        $base
    };
}
generate_alias!(pub type Extend <A B C D E F G H J K L M N O P Q R> (Base));

#[macro_export]
macro_rules! Error {
    [($base:ty) $first:ty $(| $rest:ty)*] => {
        crate::Error![
            (<$base as crate::error::SetKind<$first>>::Res)
            $($rest)|*
        ]
    };
    [($base:ty) ] => { $base };
    [ $(kinds:ty)|* ] => { crate::Error![ (crate::error::Error) $($kinds)|* ]}
}

pub trait Require<T: IsKind> = GetTypeOf<T> where <Self as GetTypeOf<T>>::Type: IsNot<!>;
pub trait Deny<T: IsKind> = GetTypeOf<T> where <Self as GetTypeOf<T>>::Type: Is<!>;

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions as sa;

    type Foo = Extend;
    type Bar = Extend<kinds::NoSuchKey<()>>;
    type Baz = Extend<kinds::NoSuchKey<()>, kinds::StorageError<u8>>;

    sa::assert_type_eq_all!(Foo, Error<>);
    sa::assert_type_eq_all!(Bar, Error<()>);
    sa::assert_type_eq_all!(Baz, Error<(), u8>);

    sa::assert_type_eq_all!(<Foo as GetTypeOf<kinds::NoSuchKey>>::Type, !);
    sa::assert_type_eq_all!(<Bar as GetTypeOf<kinds::NoSuchKey>>::Type, ());
    sa::assert_type_eq_all!(<Baz as GetTypeOf<kinds::StorageError>>::Type, u8);
}
