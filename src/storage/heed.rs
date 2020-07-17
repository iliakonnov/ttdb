use crate::api::{Database, CanRead, CanWrite, Storage, GetError, SetError, RemoveError};
use heed::types::OwnedSlice;
use heed::EnvOpenOptions;
use std::path::Path;
use std::fs;

type Data = heed::Database<OwnedSlice<u8>, OwnedSlice<u8>>;

struct Databases {
    data: Data,
    children: Data,
}

struct HeedDb {
    env: heed::Env,
    dbs: Databases
}

#[derive(Copy, Clone)]
struct Transaction<'db, T> where T: 'db {
    txn: T,
    dbs: &'db Databases
}

impl HeedDb {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        fs::create_dir_all(&path).map_err(|_| ())?;
        let env = EnvOpenOptions::new().open(path).map_err(|_| ())?;
        let storage = env.create_database(Some("storage")).map_err(|_| ())?;
        let children = env.create_database(Some("children")).map_err(|_| ())?;
        Ok(HeedDb {
            env,
            dbs: Databases {
                data: storage,
                children
            }
        })
    }
}

impl<'db> Database<'db> for HeedDb {
    type RoTxn = Transaction<'db, heed::RoTxn>;
    type RwTxn = Transaction<'db, heed::RwTxn<'db>>;

    fn ro(&'db self) -> Self::RoTxn {
        Transaction {
            txn: self.env.read_txn().unwrap(),
            dbs: &self.dbs
        }
    }

    fn rw(&'db self) -> Self::RwTxn {
        Transaction {
            txn: self.env.write_txn().unwrap(),
            dbs: &self.dbs
        }
    }
}

trait Readable {
    fn readable(&self) -> &heed::RoTxn;
}
impl Readable for heed::RoTxn {
    fn readable(&self) -> &heed::RoTxn {self}
}
impl<'a> Readable for heed::RwTxn<'a> {
    fn readable(&self) -> &heed::RoTxn {self}
}

impl Storage {
    fn get_db(self, dbs: &Databases) -> &Data {
        match self {
            Storage::Data => &dbs.data,
            Storage::Children => &dbs.children,
        }
    }
}

impl<'db, T: Readable> CanRead for Transaction<'db, T> {
    type ExistsErr = heed::Error;
    fn exists(&self, storage: Storage, path: &[u8]) -> Result<bool, Self::ExistsErr> {
        match self.get(storage, path) {
            Ok(_) => Ok(true),
            Err(GetError::NoSuchPath) => Ok(false),
            Err(GetError::Other(e)) => Err(e),
            Err(GetError::DeserializationError(_)) => unreachable!()
        }
    }

    type GetErr = heed::Error;
    fn get(&self, storage: Storage, path: &[u8]) -> Result<Vec<u8>, GetError<Self::GetErr>> {
        let res = storage.get_db(self.dbs)
            .get(self.txn.readable(), path);
        match res {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(GetError::NoSuchPath),
            Err(e) => Err(GetError::Other(e))
        }
    }
}

impl<'db> CanWrite for Transaction<'db, heed::RwTxn<'db>> {
    type SetErr = heed::Error;
    fn set(&mut self, storage: Storage, path: &[u8], data: &[u8]) -> Result<(), SetError<Self::SetErr>> {
        // FIXME: Test for parent to exist
        storage.get_db(self.dbs)
            .put(&mut self.txn, path, data)
            .map_err(SetError::Other)?;
        Ok(())
    }

    type RemoveErr = heed::Error;
    fn remove(&mut self, storage: Storage, path: &[u8]) -> Result<(), RemoveError<Self::RemoveErr>> {
        let res = storage.get_db(self.dbs)
            .delete(&mut self.txn, path)
            .map_err(RemoveError::Other)?;
        if res {
            Ok(())
        } else {
            Err( RemoveError::NoSuchPath)
        }
    }
}
