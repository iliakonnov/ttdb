#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Storage {
    Data,
    Children,
}

pub trait Database<'db>: Sized {
    type RoTxn: CanRead + 'db;
    type RwTxn: CanWrite + 'db;
    fn ro(&'db self) -> Self::RoTxn;
    fn rw(&'db self) -> Self::RwTxn;
}

#[derive(Debug)]
pub enum GetError<T> {
    NoSuchPath,
    DeserializationError(Box<dyn std::error::Error>),
    Other(T)
}

pub trait CanRead {
    type ExistsErr;
    fn exists(&self, storage: Storage, path: &[u8]) -> Result<bool, Self::ExistsErr>;

    type GetErr;
    fn get(&self, storage: Storage, path: &[u8]) -> Result<Vec<u8>, GetError<Self::GetErr>>;
}

#[derive(Debug)]
pub enum SetError<T> {
    NoParentExists,
    SerializationError(Box<dyn std::error::Error>),
    Other(T)
}
#[derive(Debug)]
pub enum RemoveError<T> {
    NoSuchPath,
    Other(T)
}
pub trait CanWrite: CanRead {
    type SetErr;
    fn set(&mut self, storage: Storage, path: &[u8], data: &[u8]) -> Result<(), SetError<Self::SetErr>>;
    type RemoveErr;
    fn remove(&mut self, storage: Storage, path: &[u8]) -> Result<(), RemoveError<Self::RemoveErr>>;
}

#[derive(Debug, Copy, Clone)]
pub struct NoTxn;
// Newtype wrappers around `Db::R?Txn`
// They are required because Rw<D> != Ro<D> != NoTxn,
// but D::RwTxn, D::RoTxn, NoTxn may equal to each other.
// So it is impossible to implement AccessExt for Access<..., Db::R?Txn> directly
// because of conflicting implementations.
#[derive(Debug)]
pub struct Ro<'db, D: Database<'db>>(pub D::RoTxn);
impl<'db, D: Database<'db>> CanRead for Ro<'db, D> {
    type ExistsErr = <<D as Database<'db>>::RoTxn as CanRead>::ExistsErr;
    fn exists(&self, storage: Storage, path: &[u8]) -> Result<bool, Self::ExistsErr> {
        self.0.exists(storage, path)
    }

    type GetErr = <<D as Database<'db>>::RoTxn as CanRead>::GetErr;
    fn get(&self, storage: Storage, path: &[u8]) -> Result<Vec<u8>, GetError<Self::GetErr>> {
        self.0.get(storage, path)
    }
}

#[derive(Debug)]
pub struct Rw<'db, D: Database<'db>>(pub D::RwTxn);
impl<'db, D: Database<'db>> CanRead for Rw<'db, D> {
    type ExistsErr = <<D as Database<'db>>::RwTxn as CanRead>::ExistsErr;
    fn exists(&self, storage: Storage, path: &[u8]) -> Result<bool, Self::ExistsErr> {
        self.0.exists(storage, path)
    }

    type GetErr = <<D as Database<'db>>::RwTxn as CanRead>::GetErr;
    fn get(&self, storage: Storage, path: &[u8]) -> Result<Vec<u8>, GetError<Self::GetErr>> {
        self.0.get(storage, path)
    }
}
impl<'db, D: Database<'db>> CanWrite for Rw<'db, D> {
    type SetErr = <<D as Database<'db>>::RwTxn as CanWrite>::SetErr;
    fn set(&mut self, storage: Storage, path: &[u8], data: &[u8]) -> Result<(), SetError<Self::SetErr>> {
        self.0.set(storage, path, data)
    }

    type RemoveErr = <<D as Database<'db>>::RwTxn as CanWrite>::RemoveErr;
    fn remove(&mut self, storage: Storage, path: &[u8]) -> Result<(), RemoveError<Self::RemoveErr>> {
        self.0.remove(storage, path)
    }
}
