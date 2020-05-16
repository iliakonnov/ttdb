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

pub trait ParentOf<Child: Path>: Path {}

pub struct Root;

impl Path for Root {
    fn into_segment(self) -> Segment {
        Segment(Vec::new())
    }
}

impl<T: Path> ParentOf<T> for Root {}

mod collect {
    use super::*;

    foldl_hlist! {
        pub trait CollectHelper |start, p: T| -> (PathBuf) where (T: Path) {
            start.0.push(p.into_segment());
            start
        }
    }

    pub trait Chain: CollectHelper {
    }

    pub fn collect<C: Chain>(chain: C) -> PathBuf {
        chain.do_foldl_hlist(PathBuf(Vec::new()))
    }
}

pub use collect::{Chain, collect as collect_chain};
