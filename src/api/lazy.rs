use super::storage::*;
use std::marker::PhantomData;
use crate::hlist::{HList, Append, Nil, Cons};
use crate::versions::Version;
use crate::path::{Path, Chain};
use crate::versions;

impl<'db, T: Database<'db>> DatabaseExt<'db> for T {}
pub trait DatabaseExt<'db>: Database<'db> {
    fn lazy(&self) -> AccessMany<Self, NoTxn, Nil> {
        AccessMany {
            db: self,
            txn: PhantomData::default(),
            result: Nil,
        }
    }
}

#[derive(Debug)]
pub struct AccessMany<'db, Db, Txn, R> {
    db: &'db Db,
    txn: PhantomData<Txn>,
    result: R,
}

pub trait ExecuteMany<Txn> {
    type Result: HList;
    fn execute(self, txn: &mut Txn) -> Self::Result;
}

impl<Txn> ExecuteMany<Txn> for Nil {
    type Result = Nil;
    fn execute(self, _txn: &mut Txn) -> Nil {
        Nil
    }
}

impl<Txn, P, R, L> ExecuteMany<Txn> for Cons<(P, R), L> where
    P: Chain,
    R: Executable<Txn>,
    L: ExecuteMany<Txn>
{
    type Result = Cons<<R as Executable<Txn>>::Result, <L as ExecuteMany<Txn>>::Result>;
    fn execute(self, txn: &mut Txn) -> Self::Result {
        let (path, res) = self.0;
        let rest = self.1;

        let path = Chain::collect(path).into_bytes();
        let res = res.execute(txn, &path);
        let rest = rest.execute(txn);
        Cons(res, rest)
    }
}

impl<'db, Db, Txn, R> AccessMany<'db, Db, Txn, R> {
    fn access<P: Chain>(self, path: P) -> Access<'db, Db, P, Txn, Nil, R> {
        Access {
            path,
            result: Nil,
            parent: self
        }
    }

    fn execute(self) -> R::Result where
        R: ExecuteMany<Txn>,
        Db: Database<'db>,
        Txn: Corresponds<'db, Db>,
    {
        let mut txn = Txn::create(self.db);
        self.result.execute(&mut txn)
    }
}

#[derive(Debug)]
pub struct Access<'db, Db, P, Txn, R, ParentRes> {
    path: P,
    result: R,
    parent: AccessMany<'db, Db, Txn, ParentRes>
}

pub trait Lazy<Txn> {
    type Result;
    fn execute(self, txn: &mut Txn, path: &[u8]) -> Self::Result;
}

pub trait Executable<Txn>: HList {
    type Result: HList;
    fn execute(self, txn: &mut Txn, path: &[u8]) -> Self::Result;
}

impl<Txn> Executable<Txn> for Nil {
    type Result = Nil;

    fn execute(self, _txn: &mut Txn, _path: &[u8]) -> Self::Result {
        Nil
    }
}
impl<Txn, T, L> Executable<Txn> for Cons<T, L> where
    T: Lazy<Txn>,
    L: Executable<Txn>
{
    type Result = Cons<T::Result, L::Result>;

    fn execute(self, txn: &mut Txn, path: &[u8]) -> Self::Result {
        Cons(self.0.execute(txn, path), self.1.execute(txn, path))
    }
}

pub trait AccessImpl<'db, Db: Database<'db>, P: Chain, Txn: Corresponds<'db, Db>, R: Executable<Txn>> {
    type NoTxn: Corresponds<'db, Db>;
    type RoTxn: Corresponds<'db, Db> + CanRead;
    type RwTxn: Corresponds<'db, Db> + CanRead + CanWrite;
}

// Ensure that AccessImpl::*Txn corresponds to Db generic parameter
pub trait Corresponds<'db, Db: Database<'db>> {
    fn create(db: &'db Db) -> Self;
}
impl<'db, Db: Database<'db>> Corresponds<'db, Db> for Ro<'db, Db> {
    fn create(db: &'db Db) -> Self {
        Ro(db.ro())
    }
}
impl<'db, Db: Database<'db>> Corresponds<'db, Db> for Rw<'db, Db> {
    fn create(db: &'db Db) -> Self {
        Rw(db.rw())
    }
}
impl<'db, Db: Database<'db>> Corresponds<'db, Db> for NoTxn {
    fn create(_db: &'db Db) -> Self {
        NoTxn
    }
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
        #[derive(Debug, Clone, Copy)]
        $vis struct $name<$($generics),*> {
            $( $arg : $ty ,)*
            $( $ignored : $ign_ty ,)*
        }
        impl<$txn_ty: $txn_bound, $($generics),*> Lazy<$txn_ty> for $name<$($generics),*>
            where $($bound)*
        {
            type Result = $res;
            fn execute(self, $txn: &mut $txn_ty, $path: &[u8]) -> Self::Result {
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
            <R as Append<$res>>::Result,
            // Leave ParentRes as is
            ParentRes
        >
    };
    ($this:expr => $lazy:expr) => {
        match $this {
            this => Access {
                path: this.path,
                parent: AccessMany {
                    db: this.parent.db,
                    result: this.parent.result,
                    // Set new txn
                    txn: PhantomData::default(),
                },
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
    | | -> Result<V, GetError<Txn::GetErr>> {
        let data = txn.get(Storage::Data, path)?;
        V::load(data).map_err(GetError::DeserializationError)
    }
);

lazy!(
    pub LazySet<V> where (V: Version + versions::Serde) {}
    Path=path
    txn=Txn: CanWrite
    | val: V | -> Result<(), SetError<Txn::SetErr>> {
        let data = val.save().map_err(SetError::SerializationError)?;
        txn.set(Storage::Data, path, &data)
    }
);

lazy!(
    pub LazyRemove<> where () {}
    Path=path
    txn=Txn: CanWrite
    | | -> Result<(), RemoveError<Txn::RemoveErr>> {
        txn.remove(Storage::Data, path)
    }
);

impl<'db, Db, P, Txn, R, ParentRes> Access<'db, Db, P, Txn, R, ParentRes> where
    Db: Database<'db>,
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

    pub fn remove(self) -> returns!(RwTxn => LazyRemove)
        where R: Append<LazyRemove>
    {
        returns!(self => LazyRemove {})
    }

    pub fn done(self) -> AccessMany<'db, Db, Txn, <ParentRes as Append<(P, R)>>::Result>
        where ParentRes: Append<(P, R)>
    {
        AccessMany {
            db: self.parent.db,
            txn: self.parent.txn,
            result: self.parent.result.append((self.path, self.result))
        }
    }

    pub fn access<NewP: Chain>(self, path: NewP) -> Access<'db, Db, NewP, Txn, Nil, <ParentRes as Append<(P, R)>>::Result>
        where ParentRes: Append<(P, R)>
    {
        self.done().access(path)
    }

    pub fn execute(self) -> <<ParentRes as Append<(P, R)>>::Result as ExecuteMany<Txn>>::Result where
        ParentRes: Append<(P, R)>,
        <ParentRes as Append<(P, R)>>::Result: ExecuteMany<Txn>,
        Db: Database<'db>,
        Txn: Corresponds<'db, Db>,
    {
        self.done().execute()
    }
}

// We can upgrade from NoTxn to Ro<Db> and from Ro<Db> to Rw<Db>, but we never will downgrade
impl<'db, Db, P, R, ParentRes> AccessImpl<'db, Db, P, NoTxn, R> for Access<'db, Db, P, NoTxn, R, ParentRes> where
    Db: Database<'db>,
    P: Chain,
    NoTxn: Corresponds<'db, Db>,
    R: Executable<NoTxn>,
{
    type NoTxn = NoTxn;
    type RoTxn = Ro<'db, Db>;
    type RwTxn = Rw<'db, Db>;
}

impl<'db, Db, P, R, ParentRes> AccessImpl<'db, Db, P, Ro<'db, Db>, R> for Access<'db, Db, P, Ro<'db, Db>, R, ParentRes> where
    Db: Database<'db>,
    P: Chain,
    Ro<'db, Db>: Corresponds<'db, Db>,
    R: Executable<Ro<'db, Db>>,
{
    type NoTxn = Ro<'db, Db>;
    type RoTxn = Ro<'db, Db>;
    type RwTxn = Rw<'db, Db>;
}

impl<'db, Db, P, R, ParentRes> AccessImpl<'db, Db, P, Rw<'db, Db>, R> for Access<'db, Db, P, Rw<'db, Db>, R, ParentRes> where
    Db: Database<'db>,
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
    use crate::path::Root;
    use crate::storage::testdb::PanicDb;
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
        Access<'db, PanicDb, HList![Root, Foo], NoTxn, Nil, Nil>
            : AccessImpl<'db, PanicDb, HList![Root, Foo], NoTxn, Nil>);
    assert_impl!(for<'db> where()
        HList![LazyGet<i32>, LazySet<i32>]: Executable<Rw<'db, PanicDb>>);

    // We are interested only in type checking this code, so there is no #[test] attribute
    fn get_and_set_ty() {
        use crate::hlist::UnwrapAll;
        let _: HList![
            HList![i32, ()], // Foo: get, set
            HList![(), String, ()] // Bar: set, get, remove
        ] = PanicDb.lazy()
            .access(hlist![Root, Foo])
                .get::<i32>()
                .set(0_i32)
            .access(hlist![Root, Bar])
                .set("Hello".to_string())
                .get::<String>()
                .remove()
            .execute()
            .unwrap_all();
    }

    path!(
        struct Foo[i32];
        struct Bar[String];
    );
    path!(Root -> {Foo}
               -> {Bar}
         );
}
