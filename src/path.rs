use std::num::NonZeroU8;
use std::hash::Hash;
use crate::reservoir::Reservoir;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Deserialize, Serialize)]
pub struct Segment(Vec<NonZeroU8>);

// TODO: Somehow put this `#[allow]` onto `#[derive(Deserialize)]`
#[allow(clippy::unsafe_derive_deserialize)] // Vec<NonZeroU8> into Vec<u8> is safe
mod allow_lint_helper {
    use super::*;
    #[allow(clippy::module_name_repetitions)]
    #[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PathBuf(pub Vec<Segment>);
}
pub use allow_lint_helper::PathBuf;

pub type ChildrenInfo = Reservoir<Segment>;

impl PathBuf {
    fn into_bytes(self) -> Vec<u8> {
        let cap: usize = self.0.iter().map(|seg| seg.0.len()).sum();
        // Because we are adding zero bytes after each segment
        let cap = cap + self.0.len();

        let mut res = Vec::with_capacity(cap);
        for i in self.0 {
            // Converting Vec<NonZeroU8> into Vec<u8>
            let mut inner = unsafe {
                // Make it sound
                static_assertions::assert_eq_size!(NonZeroU8, u8);
                static_assertions::assert_eq_align!(NonZeroU8, u8);

                // Split vec, because we can't just transmute it.
                // Hopefully into_raw_parts takes care of not dropping underlying memory
                let (ptr, len, cap) = i.0.into_raw_parts();

                // Rebuild vec back! Finally the unsafe thing.
                let ptr = ptr as *mut u8;
                Vec::from_raw_parts(ptr, len, cap)
            };
            res.append(&mut inner);
            res.push(0);
        }
        debug_assert_eq!(res.capacity(), cap);  // No reallocation happened
        debug_assert_eq!(res.len(), res.capacity());  // And we calculated right capacity

        res
    }
}

pub trait Path: Sized {
    type AssociatedData: FirstVersion;

    fn into_segment(self) -> Segment;

    type Error: std::fmt::Debug;
    /// # Errors
    /// When it is impossible to deserialize from given segment
    fn from_segment(seg: Segment) -> Result<Self, Self::Error>;
}

pub trait ParentOf<Child: Path + ?Sized>: Path {}

mod collect {
    use super::*;
    use crate::hlist::*;

    // CollectHelper implemented only for HLists of Paths
    foldl_hlist! {
        pub trait CollectHelper |start, p: T| -> (PathBuf) where (T: Path) {
            start.0.push(p.into_segment());
            start
        }
    }

    // Does not guarantees that starts with Root
    pub trait WeakChain: CollectHelper {
    }

    impl<P> WeakChain for Cons<P, Nil> where P: Path
    {}

    impl<P, C, R> WeakChain for Cons<P, Cons<C, R>> where
        P: ParentOf<C>,
        C: Path,
        Cons<C, R>: WeakChain  // This bound is why WeakChain trait is required
    {}

    #[allow(clippy::doc_markdown)]
    /// HList of Paths which starts with Root and each is ParenOf next path.
    pub trait Chain: WeakChain {
    }

    impl<T> Chain for Cons<Root, T> where Cons<Root, T>: WeakChain {
    }

    pub fn collect<C: Chain>(chain: C) -> PathBuf {
        chain.do_foldl_hlist(PathBuf(Vec::new()))
    }
}

pub use collect::{Chain, collect as collect_chain};
use crate::versions::FirstVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnexpectedTag;

/// Important root path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Root;

impl Path for Root {
    type AssociatedData = !;

    fn into_segment(self) -> Segment {
        Segment(Vec::new())
    }

    type Error = UnexpectedTag;

    fn from_segment(seg: Segment) -> Result<Self, Self::Error> {
        if seg.0.is_empty() {
            Ok(Self)
        } else {
            Err(UnexpectedTag)
        }
    }
}

/// Path that can contain anything. Can be placed at the root. Can be followed by anything
#[derive(Debug, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub struct Any(Segment);
impl Path for Any {
    type AssociatedData = !;

    fn into_segment(self) -> Segment {
        self.0
    }

    type Error = !;

    fn from_segment(seg: Segment) -> Result<Self, Self::Error> {
        Ok(Self(seg))
    }
}

// Anything can follow Any
impl<T: Path> ParentOf<T> for Any {}
impl ParentOf<Any> for Root {}

#[macro_export]
macro_rules! path {
    ($($vis:vis struct $id:ident $([$assoc:ty])?;)+) => {
        $(
            #[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, PartialEq, Eq)]
            $vis struct $id;
            $crate::path!(@impl for $id $(with $assoc)?);
        )+
    };
    (@impl for $id:ident) => {
        $crate::path!(@impl for $id with !);
    };
    (@impl for $id:ident with $data:ty) => {
        impl $id {
            #[allow(trivial_casts)]
            const TAG: &'static [::std::num::NonZeroU8] = unsafe {
                // stringify!($ident) cannot contain NUL symbol
                // so string does not contains zero bytes. (https://stackoverflow.com/a/6907327)
                // Also NonZeroU8 guarantees to have same layout as plain u8
                // so &[u8] can be safely casted into &[NonZeroU8].
                & *(
                    stringify!($id).as_bytes()
                    as *const [u8] as *const [::std::num::NonZeroU8]
                )
            };
        }

        impl $crate::path::Path for $id {
            type AssociatedData = $data;

            fn into_segment(self) -> Segment {
                Segment(Self::TAG.to_vec())
            }

            type Error = UnexpectedTag;

            fn from_segment(seg: Segment) -> Result<Self, Self::Error> {
                if seg.0 == Self::TAG {
                    Ok(Self)
                } else {
                    Err(UnexpectedTag)
                }
            }
        }
    };
    ($parent:ident $(
        -> {$child:ident $($rest:tt)*}
    )*) => {
        $(
            $crate::path!($parent -> $child);
            $crate::path!($child $($rest)*);
        )*
    };
    ($id:ident) => {};
    ($parent:ident -> $child:ident) => {
        impl $crate::path::ParentOf<$child> for $parent {}
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use static_assertions as sa;

    path! {
        struct Foo[u8];
        pub struct Bar;
        pub(super) struct Baz;
        pub(in crate::path::test) struct Quax;
        struct Spam;
        struct Eggs;
    }
    sa::assert_type_eq_all!(<Foo as Path>::AssociatedData, u8);
    sa::assert_type_eq_all!(<Bar as Path>::AssociatedData, !);

    path! {
        Root
        -> {Foo
            -> {Bar
                -> {Baz}
                -> {Quax}
            }
            -> {Spam -> Eggs}
        }
    }
    sa::assert_impl_all!(Root: ParentOf<Foo>);
    sa::assert_impl_all!(Foo: ParentOf<Bar>);
    sa::assert_impl_all!(Bar: ParentOf<Baz>);
    sa::assert_impl_all!(Bar: ParentOf<Quax>);
    sa::assert_impl_all!(Foo: ParentOf<Spam>);
    sa::assert_impl_all!(Spam: ParentOf<Eggs>);

    sa::assert_impl_all!(HList![Root]: Chain);
    sa::assert_impl_all!(HList![Root, Foo]: Chain);

    // Foo -/> Baz
    sa::assert_not_impl_all!(HList![Root, Foo, Baz]: Chain);
    // Root -/> Bar
    sa::assert_not_impl_all!(HList![Root, Bar]: Chain);
    // Chain should start with Root
    sa::assert_not_impl_all!(HList![Foo, Bar]: Chain);

    #[test]
    fn tag() {
        let expected = b"Foo"
            .iter()
            .map(|x| NonZeroU8::new(*x).unwrap())
            .collect::<Box<[_]>>();
        assert_eq!(Foo::TAG, &expected[..]);
    }
}
