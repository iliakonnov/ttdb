// TODO: Write docs

pub mod prelude {
    pub use super::kinds as err_kinds;
    pub use super::errors;
    pub use super::ResultExt as _;
    pub use super::results;
    pub use super::Error;
}

pub trait Toggle<ErrorKind, ErrorType> {
    type Toggled;
}

pub trait ToggleDefault {
    type Default;
    fn default_err() -> Self::Default;
}

pub trait Ignore<ErrorKind> {
    type Result;
    fn ignore(self) -> Self::Result;
}

pub trait ResultExt: Sized {
    fn ignore<ErrorKind>(self) -> <Self as Ignore<ErrorKind>>::Result where Self: Ignore<ErrorKind> {
        <Self as Ignore<ErrorKind>>::ignore(self)
    }
}

impl<T, E> ResultExt for Result<T, E> {}

macro_rules! errors {
    {
        pub mod kinds;
        pub mod errors;
        pub enum Error {
        $(
            $(#[$($variant_attr:tt)*])?
            $err:ident <$t:ident $(( $($bounds:tt)*))?> {
                $($def:tt)*
            }
        ),* $(,)?
    }} => {
        use thiserror::Error;

        #[derive(Debug, Clone, Copy, Error)]
        pub enum Error<$(
            $t: kinds::$err = !
        ),*> {
            $(
                $(#[$($variant_attr)*])?
                $err($t)
            ),*
        }

        pub mod kinds {
            use std::error::Error as StdError;
            use thiserror::Error;
            $(
                pub trait $err: 'static + std::fmt::Debug $(+ $($bounds)*)? {
                }
                impl $err for ! {}

                errors!(@impl($err) $($def)*);
            )*
        }

        pub mod errors {
            errors!(@errors [] $($t: $err {$($def)*})*);
        }
    };
    {@impl($trait:ident) pub struct $(#[$($struct_attr:tt)*])? $name:ident } => {
        #[derive(Error, Clone, Copy, Debug, Default)]
        $(#[$($struct_attr)*])?
        pub struct $name;
        impl $trait for $name {}
        impl super::ToggleDefault for super::errors::$trait<!> {
            type Default = $name;
            fn default_err() -> Self::Default { $name }
        }
    };
    {@impl($trait:ident) impl $t:ty} => {
        impl $trait for $t where $t: Sized {}
    };
    {@impl($trait:ident) any} => {
        impl<T> $trait for T where T: 'static + StdError {}
    };
    {@impl($trait:ident)} => {};
    {@errors [$($before:ident : $bef_trait:ident)*]} => {};
    {@errors [$($before:ident : $bef_trait:ident)*]
         $t:ident : $trait:ident {$($def:tt)*}
         $($after:ident : $aft_trait:ident {$($aft_def:tt)*})*
    } => {
        #[derive(Debug, Clone, Copy)]
        #[allow(unreachable_code)]
        pub struct $trait<T: super::kinds::$trait = !>(pub T);
        impl<T: super::kinds::$trait> $trait<T> where
            Self: super::ToggleDefault,
            <Self as super::ToggleDefault>::Default: super::kinds::$trait
        {
            #[must_use]
            pub fn err<E: From<$trait<<Self as super::ToggleDefault>::Default>>>() -> E
            {
                $trait(<Self as super::ToggleDefault>::default_err()).into()
            }
        }
        impl<
            OkType: std::default::Default,
            $($before: super::kinds::$bef_trait ,)*
            $t: super::kinds::$trait
            $(, $after: super::kinds::$aft_trait)*
        > super::Ignore<$trait<!>> for Result<OkType, super::Error<
            $($before ,)* $t $(, $after)*
        >> {
            type Result = Result<OkType, super::Error<
                $($before ,)* ! $(, $after)*>
            >;
            fn ignore(self) -> Self::Result {
                match self {
                    Ok(x) => Ok(x),
                    Err(super::Error::$trait(_)) => Ok(std::default::Default::default()),
                    // Left `Error::*` and right `Error::*` are not same types!
                    // They differ in generic arguments of Error<...>
                    $(Err(super::Error::$bef_trait(x)) => Err(super::Error::$bef_trait(x)),)*
                    $(Err(super::Error::$aft_trait(x)) => Err(super::Error::$aft_trait(x)),)*
                }
            }
        }
        impl<
            $($before: super::kinds::$bef_trait ,)*
            $t: super::kinds::$trait
            $(, $after: super::kinds::$aft_trait)*
        > From<$trait<$t>> for super::Error<
            $($before ,)* $t $(, $after)*
        > {
            fn from(x: $trait<$t>) -> Self {
                Self::$trait(x.0)
            }
        }
        impl<
            $($before: super::kinds::$bef_trait ,)*
            $t: super::kinds::$trait, Toggled: super::kinds::$trait
            $(, $after: super::kinds::$aft_trait)*
        > super::Toggle <$trait<!>, Toggled> for super::Error<
            $($before ,)* $t $(, $after)*
        > {
            type Toggled = super::Error<$($before ,)* Toggled $(, $after)*>;
        }
        errors!(@errors
            [$($before : $bef_trait)* $t : $trait]
            $($after : $aft_trait {$($aft_def)*})*
        );
    };
}

#[macro_export]
#[allow(non_snake_case)]
macro_rules! Error {
    ([$res:ty] $(|)?) => {$res};
    ([$res:ty] ~ $id:ident | $($rest:tt)*) => {
        $crate::Error!([$res] $id
            < ! >
            | $($rest)*
        )
    };
    ([$res:ty] $id:ident<$arg:ty> | $($rest:tt)*) => {
        $crate::Error!([
            <$res as
                $crate::error::Toggle<$crate::error::errors::$id<!>, $arg>
            >::Toggled
        ] $($rest)*)
    };
    ([$res:ty] $id:ident | $($rest:tt)*) => {
        $crate::Error!([$res] $id
            < <$crate::error::errors::$id<!> as $crate::error::ToggleDefault>::Default >
            | $($rest)*
        )
    };
    ([$res:ty] $($tt:tt)*) => {
        compile_error!(concat!("No rules expected: ", stringify!($($tt)*)))
    };
    ($($tt:tt)*) => {
        $crate::Error!([$crate::error::Error] $( $tt )* |)
    }
}

errors! {
    pub mod kinds;
    pub mod errors;
    pub enum Error {
        #[error(transparent)] KeyNotFound<T1 (StdError)> {
            pub struct #[error("key not found")] KeyNotFoundError
        },
        #[error("unknown storage error")] Storage<T2> {any},
        #[error("unknown serialization error")] Serialization<T3> {},
    }
}

pub mod results {
    use crate::api::{CanRead, CanWrite};
    pub type SetResult<R, T> = Result<R, Error![
        Storage< <T as CanWrite>::SetErr >
    ]>;
    pub type RemoveResult<R, T> = Result<R, Error![
        KeyNotFound
        | Storage< <T as CanWrite>::RemoveErr >
    ]>;
    pub type GetResult<R, T> = Result<R, Error![
        KeyNotFound
        | Storage< <T as CanRead>::GetErr >
    ]>;
}

#[cfg(test)]
mod tests {
    use thiserror::Error;
    use super::prelude::*;

    #[derive(Error, Debug)]
    #[error("Test error")]
    struct Foo;

    #[allow(clippy::missing_const_for_fn)]
    fn zero() {
        let err: Option<Error![]> = None;
        match err {
            None => {},
            // We can omit the `Some(_)` pattern. Thanks to #![feature(exhaustive_patterns)]
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn one() {
        let err: Error![KeyNotFound] = errors::KeyNotFound::err();
        match err {
            super::Error::KeyNotFound(_) => {},
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn two() {
        let err: Error![KeyNotFound | Storage<Foo>] = errors::KeyNotFound::err();
        match err {
            super::Error::KeyNotFound(_) | super::Error::Storage(_) => {},
        }
    }

    #[test]
    fn ignore() {
        type Before = Result<u8, Error![KeyNotFound | Storage<Foo>]>;
        type After = Result<u8, Error![KeyNotFound]>;
        let res: Before = Ok(42);
        let foo: After = res.ignore::<errors::Storage>();
        assert_eq!(foo.ok(), Some(42));
    }

    #[test]
    fn ignore_default() {
        type Before = Result<u8, Error![KeyNotFound | Storage<Foo>]>;
        type After = Result<u8, Error![Storage<Foo>]>;
        let res: Before = Err(errors::KeyNotFound::err());
        let foo: After = res.ignore::<errors::KeyNotFound>();
        assert_eq!(foo.ok(), Some(0));
    }
}
