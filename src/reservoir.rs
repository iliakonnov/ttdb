use rand::rngs::SmallRng;
use rand::{SeedableRng, Rng};
use indexmap::IndexSet;
use std::hash::Hash;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReservoirSize {
    All,
    Maximum(usize)
}

#[derive(Clone)]
pub enum Reservoir<T> {
    Limited(LimitedReservoir<T>),
    Unlimited {
        buf: IndexSet<T>,
    }
}

impl<T: Hash + Eq> Reservoir<T> {
    pub fn new(size: ReservoirSize, buf: IndexSet<T>) -> Self {
        match size {
            ReservoirSize::All => {
                Reservoir::Unlimited {
                    buf
                }
            },
            ReservoirSize::Maximum(max) => {
                Reservoir::Limited(LimitedReservoir::new(max, buf))
            },
        }
    }

    pub fn insert(&mut self, val: T) -> InsertionResult<T> {
        match self {
            Reservoir::Unlimited { buf } => {
                if buf.insert(val) {
                    InsertionResult::Overwritten
                } else {
                    InsertionResult::Inserted
                }
            },
            Reservoir::Limited(lim) => {
                lim.insert(val)
            },
        }
    }

    pub fn inner(&self) -> &IndexSet<T> {
        match self {
            Reservoir::Limited(lim) => lim.inner(),
            Reservoir::Unlimited { buf } => buf,
        }
    }
}

#[derive(Clone)]
pub struct LimitedReservoir<T> {
    rng: SmallRng,
    buf: IndexSet<T>,
    fullness: usize,  // How many space remaining until hitting cap
    total_count: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RestoreError {
    /// When buf.len() is more than state.max
    TooBig
}

#[derive(Debug, Copy, Clone)]
pub enum InsertionResult<T> {
    // When same item already exists in set
    Overwritten,
    // When some item was removed
    Replaced {
        removed: T
    },
    // When new item was not inserted at all
    Dropped {
        val: T
    },
    // When there was enough space
    Inserted
}

impl<T: Hash + Eq> LimitedReservoir<T> {
    pub fn new(max: usize, buf: IndexSet<T>) -> Self {
        let total_count = buf.len();
        Self::with_count(max, buf, total_count)
    }

    pub fn with_count(max: usize, mut buf: IndexSet<T>, total_count: usize) -> Self {
        let mut rng = SmallRng::from_rng(rand::thread_rng()).unwrap();

        let fullness = max.saturating_sub(buf.len());
        for _ in max..buf.len() {
            buf.swap_remove_index(rng.gen_range(0, buf.len()));
        }

        Self {
            rng,
            buf,
            fullness,
            total_count
        }
    }

    pub fn inner(&self) -> &IndexSet<T> {
        &self.buf
    }

    pub fn insert(&mut self, val: T) -> InsertionResult<T> {
        self.total_count += 1;
        match self.fullness {
            0 => self.replace(val),
            _ => self.append(val)
        }
    }

    pub fn max_size(&self) -> usize {
        self.buf.len() + self.fullness
    }

    fn replace(&mut self, val: T) -> InsertionResult<T> {
        debug_assert_eq!(self.fullness, 0);
        // Replace random item
        let idx = self.rng.gen_range(0, self.total_count - 1);
        if idx < self.buf.len() {
            // Replace item
            let removed = self.buf.swap_remove_index(idx).unwrap();
            self.buf.insert(val);
            InsertionResult::Replaced {
                removed
            }
        } else {
            InsertionResult::Dropped {
                val
            }
        }
    }

    fn append(&mut self, val: T) -> InsertionResult<T> {
        debug_assert_ne!(self.fullness, 0);
        self.buf.insert(val);
        self.fullness -= 1;
        InsertionResult::Inserted
    }
}

#[cfg(test)]
mod test {
    // TODO: Not enough tests
    use super::*;
    use fntools::value::ValueExt;

    #[test]
    fn empty() {
        let empty = LimitedReservoir::new(42, IndexSet::<u8>::new());
        assert!(empty.inner().is_empty());
        assert_eq!(empty.max_size(), 42);
        assert_eq!(empty.total_count, 0);  // No items inserted
        assert_eq!(empty.fullness, 42);  // So there are 42 places remaining
    }

    #[test]
    fn insert_one() {
        let mut empty = LimitedReservoir::new(42, IndexSet::new());
        assert!(matches!(empty.insert(7), InsertionResult::Inserted));
        assert_eq!(empty.inner().iter().collect::<Vec<_>>(), vec![&7]);
        assert_eq!(empty.max_size(), 42);
        assert_eq!(empty.total_count, 1);
        assert_eq!(empty.fullness, 41);
    }

    #[test]
    fn overfull() {
        // FIXME: Too complicated test because of randomness
        let mut empty = LimitedReservoir::new(3, IndexSet::new());
        assert_eq!((empty.total_count, empty.fullness), (0, 3));

        assert!(matches!(empty.insert(3), InsertionResult::Inserted));
        assert_eq!((empty.total_count, empty.fullness), (1, 2));

        assert!(matches!(empty.insert(7), InsertionResult::Inserted));
        assert_eq!((empty.total_count, empty.fullness), (2, 1));

        assert!(matches!(empty.insert(13), InsertionResult::Inserted));
        assert_eq!((empty.total_count, empty.fullness), (3, 0));

        let inserted = empty.insert(42);
        assert_eq!((empty.total_count, empty.fullness), (4, 0));

        // Sort because Reservoir don't care about IndexSet order. It is unspecified.
        let collected = empty.inner().iter().copied().collect::<Vec<_>>().also(|t| t.sort());
        match inserted {
            InsertionResult::Dropped { val } => {
                assert_eq!(val, 42);
                assert_eq!(collected, vec![3, 7, 13]);
            },
            InsertionResult::Replaced { removed } => {
                match removed {
                    3 => assert_eq!(collected, vec![7, 13, 42]),
                    7 => assert_eq!(collected, vec![3, 13, 42]),
                    13 => assert_eq!(collected, vec![3, 7, 42]),
                    x => panic!("empty.insert replaced invalid value: {:?}", x)
                }
            },
            x => panic!("empty.insert returned invalid value: {:?}", x)
        }
    }
}
