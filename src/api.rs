use std::marker::PhantomData;
use crate::hlist::{HList, Append, Nil, Cons};
use crate::versions::Version;
use crate::path::{Path, ChildrenInfo, Chain};
use crate::versions;
use derivative::Derivative;

pub trait Database: Sized {
    type RoTxn: CanRead;
    type RwTxn: CanWrite;
    fn ro(&self) -> &Self::RoTxn;
    fn rw(&self) -> &Self::RwTxn;
}

impl<T: Database> DatabaseExt for T {}
pub trait DatabaseExt: Database {
    fn access<P: Chain>(&self, path: P) -> Access<Self, P, NoTxn, Nil> {
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

#[derive(Debug)]
pub struct Access<'db, Db, P, Txn, R> {
    db: &'db Db,
    path: P,
    txn: PhantomData<Txn>,
    result: R,
}

pub trait Lazy<Txn> {
    type Result;
    fn execute(self, txn: Txn, path: &[u8]) -> Self::Result;
}

pub trait Executable<Txn>: HList {
    type Result: HList;
    fn execute(self, txn: Txn, path: &[u8]) -> Self::Result;
}

impl<Txn> Executable<Txn> for Nil {
    type Result = Nil;

    fn execute(self, _txn: Txn, _path: &[u8]) -> Self::Result {
        Nil
    }
}
impl<Txn, T, L> Executable<Txn> for Cons<T, L> where
    // Require copy, so we can not to take reference to R?<Db> which is wrapper around reference too
    Txn: Copy,
    T: Lazy<Txn>,
    L: Executable<Txn>
{
    type Result = Cons<T::Result, L::Result>;

    fn execute(self, txn: Txn, path: &[u8]) -> Self::Result {
        Cons(self.0.execute(txn, path), self.1.execute(txn, path))
    }
}

// Ensure that AccessImpl::*Txn corresponds to Db generic parameter
pub trait Corresponds<'db, Db: Database> {
    fn create(db: &'db Db) -> Self;
}
impl<'db, Db: Database> Corresponds<'db, Db> for Ro<'db, Db> {
    fn create(db: &'db Db) -> Self {
        Ro(db.ro())
    }
}
impl<'db, Db: Database> Corresponds<'db, Db> for Rw<'db, Db> {
    fn create(db: &'db Db) -> Self {
        Rw(db.rw())
    }
}
impl<'db, Db: Database> Corresponds<'db, Db> for NoTxn {
    fn create(_db: &'db Db) -> Self {
        NoTxn
    }
}

pub trait AccessImpl<'db, Db: Database, P: Chain, Txn: Corresponds<'db, Db>, R: Executable<Txn>> {
    type NoTxn: Corresponds<'db, Db>;
    type RoTxn: Corresponds<'db, Db> + CanRead;
    type RwTxn: Corresponds<'db, Db> + CanRead + CanWrite;
}

// Creates new struct, that implements Lazy<Txn>
macro_rules! lazy {
    ($vis:vis $name:ident <$($generics:ident),*> where ($($bound:tt)*)
        { $($ignored:ident: $ign_ty:ty),* }
     Path=$path:ident
     $txn:ident=$txn_ty:ident : $txn_bound:path
     |$($arg:ident : $ty:ty),*| -> $res:ty {
        $($body:tt)*
     }
    ) => {
        #[derive(Debug)]
        $vis struct $name<$($generics),*> {
            $( $arg : $ty ,)*
            $( $ignored : $ign_ty ,)*
        }
        impl<$txn_ty: $txn_bound, $($generics),*> Lazy<$txn_ty> for $name<$($generics),*>
            where $($bound)*
        {
            type Result = $res;
            fn execute(self, $txn: $txn_ty, $path: &[u8]) -> Self::Result {
                let Self { $($arg,)* .. } = self;
                $($body)*
            }
        }
    };
}

// These types are so complex that I made special macro for them...
macro_rules! returns {
    ($txn:ident => $res:ty) => {
        Access<
            // Some things not changed
            'db, Db, P,
            // Set new Txn
            <Self as AccessImpl<'db, Db, P, Txn, R>>::$txn,
            // Append $res to return type
            <R as Append<$res>>::Result
        >
    };
    ($this:expr => $lazy:expr) => {
        match $this {
            this => Access {
                // Not changed
                db: this.db,
                path: this.path,
                // Set new txn,
                txn: PhantomData::default(),
                // Append $lazy to result
                result: this.result.append($lazy),
            }
        }
    };
}

lazy!(
    pub LazyGet<V> where (V: Version + versions::Serde) { phantom: PhantomData<V> }
    Path=path
    txn=Txn: CanRead
    | | -> V {
        let _ = (path, txn);
        todo!()
    }
);

lazy!(
    pub LazySet<V> where (V: Version + versions::Serde) {}
    Path=path
    txn=Txn: CanWrite
    | val: V | -> () {
        let _ = (val, path, txn);
        todo!()
    }
);

impl<'db, Db, P, Txn, R> Access<'db, Db, P, Txn, R> where
    Db: Database,
    P: Chain,
    Txn: Corresponds<'db, Db>,
    R: Executable<Txn>,
    Self: AccessImpl<'db, Db, P, Txn, R>
{
    pub fn get<V>(self) -> returns!(RoTxn => LazyGet<V>) where
        R: Append<LazyGet<V>>,
        V: Version<FirstVersion=<<P as Chain>::Last as Path>::AssociatedData> + versions::Serde
    {
        returns!(self => LazyGet {
            phantom: PhantomData::default()
        })
    }

    pub fn set<V>(self, val: V) -> returns!(RwTxn => LazySet<V>) where
        R: Append<LazySet<V>>,
        V: Version<FirstVersion=<<P as Chain>::Last as Path>::AssociatedData> + versions::Serde
    {
        returns!(self => LazySet {
            val
        })
    }

    pub fn execute(self) -> R::Result {
        let txn = Txn::create(self.db);
        let path = Chain::collect(self.path).into_bytes();
        self.result.execute(txn, &path)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NoTxn;
// Newtype wrappers around `Db::R?Txn`
// They are required because Rw<D> != Ro<D> != NoTxn,
// but D::RwTxn, D::RoTxn, NoTxn may equal to each other.
// So it is impossible to implement AccessExt for Access<..., Db::R?Txn> directly
// because of conflicting implementations.
#[derive(Debug, Derivative)]
#[derivative(Copy, Clone)]
pub struct Ro<'db, D: Database>(&'db D::RoTxn);
impl<'db, D: Database> CanRead for Ro<'db, D> {
    fn get(&self, path: &[u8]) -> Vec<u8> {
        self.0.get(path)
    }

    fn get_children(&self, path: &[u8]) -> ChildrenInfo {
        self.0.get_children(path)
    }
}

#[derive(Debug, Derivative)]
#[derivative(Copy, Clone)]
pub struct Rw<'db, D: Database>(&'db D::RwTxn);
impl<'db, D: Database> CanRead for Rw<'db, D> {
    fn get(&self, path: &[u8]) -> Vec<u8> {
        self.0.get(path)
    }

    fn get_children(&self, path: &[u8]) -> ChildrenInfo {
        self.0.get_children(path)
    }
}
impl<'db, D: Database> CanWrite for Rw<'db, D> {
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
impl<'db, Db, P, R> AccessImpl<'db, Db, P, NoTxn, R> for Access<'db, Db, P, NoTxn, R> where
    Db: Database,
    P: Chain,
    NoTxn: Corresponds<'db, Db>,
    R: Executable<NoTxn>,
{
    type NoTxn = NoTxn;
    type RoTxn = Ro<'db, Db>;
    type RwTxn = Rw<'db, Db>;
}

impl<'db, Db, P, R> AccessImpl<'db, Db, P, Ro<'db, Db>, R> for Access<'db, Db, P, Ro<'db, Db>, R> where
    Db: Database,
    P: Chain,
    Ro<'db, Db>: Corresponds<'db, Db>,
    R: Executable<Ro<'db, Db>>,
{
    type NoTxn = Ro<'db, Db>;
    type RoTxn = Ro<'db, Db>;
    type RwTxn = Rw<'db, Db>;
}

impl<'db, Db, P, R> AccessImpl<'db, Db, P, Rw<'db, Db>, R> for Access<'db, Db, P, Rw<'db, Db>, R> where
    Db: Database,
    P: Chain,
    Rw<'db, Db>: Corresponds<'db, Db>,
    R: Executable<Rw<'db, Db>>,
{
    type NoTxn = Rw<'db, Db>;
    type RoTxn = Rw<'db, Db>;
    type RwTxn = Rw<'db, Db>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::{ChildrenInfo, Root};
    extern crate static_assertions as sa;

    // Unfortunately static_assertions does not support generics
    // Even [commit] does not allow generics in bounds
    // [commit]: https://github.com/nvzqz/static-assertions-rs/commit/87c22afad1a7f945dd0fc424658b99388d4bc109
    macro_rules! assert_impl {
        (
            for<
                $($lifetime:lifetime),* $(,)?
                $($generic:ident),* $(,)?
            >
            where($($bound:tt)*)
            $type:ty : $($trait:tt)*
        ) => {
            const _: fn() = || {
                // Check that __T implements trait
                fn assert_impl_all<$($lifetime,)* $($generic,)* __T: ?Sized + $($trait)*>()
                where $($bound)* {}

                // Introduce generics and try to call assert_impl_all
                fn foo<$($lifetime,)* $($generic,)*>() where $($bound)* {
                    assert_impl_all::<$($lifetime,)* $($generic,)* $type>();
                }
            };
        };
    }

    assert_impl!(for<'db> where()
        Access<'db, MockDb, HList![Root, Foo], NoTxn, Nil>
            : AccessImpl<'db, MockDb, HList![Root, Foo], NoTxn, Nil>);
    assert_impl!(for<'db> where()
        HList![LazyGet<i32>, LazySet<i32>]: Executable<Rw<'db, MockDb>>);

    // We are interested only in type checking this code, so there is no #[test] attribute
    fn get_and_set_ty() {
        let _: HList![i32, ()] = MockDb(MockTxn).access(hlist![Root, Foo])
            .get::<i32>()
            .set(0_i32)
            .execute();
    }

    path!(struct Foo[i32];);
    path!(Root -> Foo);

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
