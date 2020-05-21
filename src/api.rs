use std::marker::PhantomData;
use crate::hlist::{HList, Nil, Cons};
use crate::versions::Version;
use crate::path::{Path, ChildrenInfo};

pub trait Database: Sized {
    type RoTxn: CanRead;
    type RwTxn: CanWrite;
    fn ro(&self) -> &Self::RoTxn;
    fn rw(&self) -> &Self::RwTxn;
}

impl<T: Database> DatabaseExt for T {}
pub trait DatabaseExt: Database {
    fn access<P: Path>(&self, path: P) -> Access<Self, P, Nil, !> {
        Access {
            db: self,
            path,
            result: Nil,
            txn: PhantomData::default()
        }
    }
}

pub trait CanRead {
    fn get(&self, path: &[u8]) -> Vec<u8>;
    fn get_children(&self, path: &[u8]) -> ChildrenInfo;
}
pub trait CanWrite: CanRead {
    fn set(&self, path: &[u8], data: Vec<u8>);
    fn set_children(&self, parent: &[u8], children: ChildrenInfo);
    fn remove(&self, path: &[u8]);
}
struct NoTxn;

#[derive(Debug)]
pub struct Access<'db, Db: Database, P: Path, R: HList, Txn> {
    db: &'db Db,
    path: P,
    result: R,
    txn: PhantomData<Txn>
}

struct LazyGet<V>(PhantomData<V>);
struct LazySet<V>(V);

trait AccessImpl<'db, Db: Database, P: Path, R: HList> {
    type NoTxn;
    type RoTxn: CanRead;
    type RwTxn: CanRead + CanWrite;
    fn get<V>(self) -> Access<'db, Db, P, Cons<LazyGet<V>, R>, Self::RoTxn>
        where V: Version<FirstVersion=P::AssociatedData>;
    fn set<V>(self, val: V) -> Access<'db, Db, P, Cons<LazySet<V>, R>, Self::RwTxn>
        where V: Version<FirstVersion=P::AssociatedData>;
}

default impl<'db, Db: Database, P: Path, R: HList, Txn> AccessImpl<'db, Db, P, R> for Access<'db, Db, P, R, Txn> {
    fn get<V>(self) -> Access<'db, Db, P, Cons<LazyGet<V>, R>, Self::RoTxn>
        where V: Version<FirstVersion=P::AssociatedData>
    {
        Access {
            db: self.db,
            path: self.path,
            result: Cons(LazyGet(PhantomData::default()), self.result),
            txn: PhantomData::default()
        }
    }
    fn set<V>(self, val: V) -> Access<'db, Db, P, Cons<LazySet<V>, R>, Self::RwTxn>
        where V: Version<FirstVersion=P::AssociatedData>
    {
        Access {
            db: self.db,
            path: self.path,
            result: Cons(LazySet(val), self.result),
            txn: PhantomData::default()
        }
    }
}

// Newtype wrappers around `Db::R?Txn`
// They are required because Rw<D> != Ro<D> != NoTxn,
// but D::RwTxn, D::RoTxn, NoTxn may equal to each other.
// So it is impossible to implement AccessExt for Access<..., Db::R?Txn> directly
// because of conflicting implementations:
/*
| impl<'db, Db: Database, P: Path, R: HList> AccessExt<'db, Db, P, R> for Access<'db, Db, P, R, NoTxn> {
| ---------------------------------------------------------------------------------------------------- first implementation here
...
| impl<'db, Db: Database, P: Path, R: HList> AccessExt<'db, Db, P, R> for Access<'db, Db, P, R, Db::RwTxn> {
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `api::Access<'_, _, _, _, api::NoTxn>`
*/
struct Ro<D: Database>(D::RoTxn);
impl<D: Database> CanRead for Ro<D> {
    fn get(&self, path: &[u8]) -> Vec<u8> {
        self.0.get(path)
    }

    fn get_children(&self, path: &[u8]) -> ChildrenInfo {
        self.0.get_children(path)
    }
}

struct Rw<D: Database>(D::RwTxn);
impl<D: Database> CanRead for Rw<D> {
    fn get(&self, path: &[u8]) -> Vec<u8> {
        self.0.get(path)
    }

    fn get_children(&self, path: &[u8]) -> ChildrenInfo {
        self.0.get_children(path)
    }
}
impl<D: Database> CanWrite for Rw<D> {
    fn set(&self, path: &[u8], data: Vec<u8>) {
        self.0.set(path, data)
    }

    fn set_children(&self, parent: &[u8], children: ChildrenInfo) {
        self.0.set_children(parent, children)
    }

    fn remove(&self, path: &[u8]) {
        self.0.remove(path)
    }
}

// We can upgrade from NoTxn to Ro<Db> and from Ro<Db> to Rw<Db>, but we never will downgrade
impl<'db, Db: Database, P: Path, R: HList> AccessImpl<'db, Db, P, R> for Access<'db, Db, P, R, NoTxn> {
    type NoTxn = NoTxn;
    type RoTxn = Ro<Db>;
    type RwTxn = Rw<Db>;
}

impl<'db, Db: Database, P: Path, R: HList> AccessImpl<'db, Db, P, R> for Access<'db, Db, P, R, Ro<Db>> {
    type NoTxn = Ro<Db>;
    type RoTxn = Ro<Db>;
    type RwTxn = Rw<Db>;
}

impl<'db, Db: Database, P: Path, R: HList> AccessImpl<'db, Db, P, R> for Access<'db, Db, P, R, Rw<Db>> {
    type NoTxn = Rw<Db>;
    type RoTxn = Rw<Db>;
    type RwTxn = Rw<Db>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::{ChildrenInfo, Root};
    extern crate static_assertions as sa;

    trait CheckImpl1<'a>: AccessImpl<'a, MockDb, Root, Nil> {}
    impl<'a> CheckImpl1<'a> for Access<'a, MockDb, Root, Nil, NoTxn> {}

    trait CheckImpl2<'a>: AccessImpl<'a, MockDb, Root, Nil> {}
    impl<'a> CheckImpl2<'a> for Access<'a, MockDb, Root, Nil, Ro<MockDb>> {}

    struct MockDb(MockTxn);
    impl Database for MockDb {
        type RoTxn = MockTxn;
        type RwTxn = MockTxn;

        fn ro(&self) -> &Self::RoTxn {
            &self.0
        }

        fn rw(&self) -> &Self::RwTxn {
            &self.0
        }
    }

    struct MockTxn;
    impl CanRead for MockTxn {
        fn get(&self, _path: &[u8]) -> Vec<u8> {
            panic!("MockTxn won't do anything")
        }

        fn get_children(&self, _path: &[u8]) -> ChildrenInfo {
            panic!("MockTxn won't do anything")
        }
    }
    impl CanWrite for MockTxn {
        fn set(&self, _path: &[u8], _data: Vec<u8>) {
            panic!("MockTxn won't do anything")
        }

        fn set_children(&self, _parent: &[u8], _children: ChildrenInfo) {
            panic!("MockTxn won't do anything")
        }

        fn remove(&self, _path: &[u8]) {
            panic!("MockTxn won't do anything")
        }
    }
}
