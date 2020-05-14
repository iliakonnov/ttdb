use std::num::NonZeroU8;
use indexmap::IndexSet;
use std::hash::Hash;
use crate::reservoir::{Reservoir, Size};
use serde::{Serialize, Deserialize};

#[derive(Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Deserialize, Serialize)]
pub struct Segment(Vec<NonZeroU8>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChildrenInfo {
    size: Size,
    children: Reservoir<Segment>
}

impl ChildrenInfo {
    pub fn get_children(&self) -> &IndexSet<Segment> {
        self.children.inner()
    }

    pub fn add_children(&mut self, segment: Segment) {
        self.children.insert(segment);
    }
}
