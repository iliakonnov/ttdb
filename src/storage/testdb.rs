use crate::api::{Database, CanRead, CanWrite, Storage, SetResult, RemoveResult, GetResult};

/// Database storage that only panic
#[derive(Copy, Clone, Debug)]
pub struct PanicDb;
#[derive(Copy, Clone, Debug)]
pub struct PanicTxn;

impl<'db> Database<'db> for PanicDb {
    type RoTxn = PanicTxn;
    type RwTxn = PanicTxn;

    fn ro(&'db self) -> Self::RoTxn {
        PanicTxn
    }

    fn rw(&'db self) -> Self::RwTxn {
        PanicTxn
    }
}

impl<'db> CanRead for PanicTxn {
    type GetErr = !;
    fn get(&self, _storage: Storage, _path: &[u8]) -> GetResult<Vec<u8>, Self> {
        panic!("PanicDb will only panic")
    }
}
impl<'db> CanWrite for PanicTxn {
    type SetErr = !;
    fn set(&mut self, _storage: Storage, _path: &[u8], _data: &[u8]) -> SetResult<(), Self> {
        panic!("PanicDb will only panic")
    }

    type RemoveErr = !;
    fn remove(&mut self, _storage: Storage, _path: &[u8]) -> RemoveResult<(), Self> {
        panic!("PanicDb will only panic")
    }
}
