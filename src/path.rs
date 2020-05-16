use std::num::NonZeroU8;
use std::hash::Hash;
use crate::reservoir::Reservoir;
use serde::{Serialize, Deserialize};

#[derive(Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Deserialize, Serialize)]
pub struct Segment(Vec<NonZeroU8>);

#[allow(clippy::module_name_repetitions)]
pub struct PathBuf(Vec<Segment>);

pub type ChildrenInfo = Reservoir<Segment>;

impl PathBuf {
    fn into_bytes(self) -> Vec<u8> {
        let cap: usize = self.0.iter().map(|seg| seg.0.len()).sum();
        // Because we are adding zero bytes after each segment
        let cap = cap + self.0.len();

        let mut res = Vec::with_capacity(cap);
        for i in self.0 {
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

pub trait Path {
    fn into_segment(self) -> Segment;
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

/// Important root path.
pub struct Root;

impl Path for Root {
    fn into_segment(self) -> Segment {
        Segment(Vec::new())
    }
}

/// Path that can contain anything. Can be placed at the root. Can be followed by anything
pub struct Any(Segment);
impl Path for Any {
    fn into_segment(self) -> Segment {
        Segment(vec![NonZeroU8::new(1).unwrap()])
    }
}

// Anything can follow Any
impl<T: Path> ParentOf<T> for Any {}
impl ParentOf<Any> for Root {}

#[macro_export]
macro_rules! path {
    // Only simple paths supported: they cannot contain any data.
    ($($vis:vis struct $id:ident;)+) => {
        $(
            $vis struct $id;
            $crate::path!(@impl for $id);
        )+
    };
    (@impl for $id:ident) => {
        impl $crate::path::Path for $id {
            fn into_segment(self) -> Segment {
                #[allow(trivial_casts)]
                const NAME: &[::std::num::NonZeroU8] = unsafe {
                    // stringify!($ident) cannot contain NUL symbol
                    // so string does not contains zero bytes. (https://stackoverflow.com/a/6907327)
                    // Also NonZeroU8 guarantees to have same layout as plain u8
                    // so &[u8] can be safely casted into &[NonZeroU8].
                    & *(
                        stringify!($id).as_bytes()
                        as *const [u8] as *const [::std::num::NonZeroU8]
                    )
                };
                Segment(NAME.to_vec())
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
        struct Foo;
        pub struct Bar;
        pub(super) struct Baz;
        pub(in crate::path::test) struct Quax;
        struct Spam;
        struct Eggs;
    }
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
}
