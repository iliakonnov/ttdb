use std::num::NonZeroU8;
use indexmap::IndexSet;
use std::hash::Hash;
use crate::reservoir::{Reservoir, ReservoirSize};
use serde::{Serialize, Deserialize};

#[derive(Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Deserialize, Serialize)]
pub struct PathSegment(Vec<NonZeroU8>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChildrenInfo {
    size: ReservoirSize,
    children: Reservoir<PathSegment>
}

impl ChildrenInfo {
    pub fn get_children(&self) -> &IndexSet<PathSegment> {
        &self.children.inner()
    }

    pub fn add_children(&mut self, segment: PathSegment) {
        self.children.insert(segment);
    }
}
