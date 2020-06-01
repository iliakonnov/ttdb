use crate::api::{Database, CanRead, CanWrite};
use crate::path::ChildrenInfo;
use heed::types::{OwnedSlice, SerdeBincode};
use heed::EnvOpenOptions;
use std::path::Path;
use std::fs;

type Storage<T=OwnedSlice<u8>> = heed::Database<OwnedSlice<u8>, T>;
type Children = Storage<SerdeBincode<ChildrenInfo>>;

struct Databases {
    storage: Storage,
    children: Children,
}

struct HeedDb {
    env: heed::Env,
    db: Databases
}

#[derive(Copy, Clone)]
struct Transaction<'db, T> where T: 'db {
    txn: T,
    db: &'db Databases
}

impl HeedDb {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        fs::create_dir_all(&path).map_err(|_| ())?;
        let env = EnvOpenOptions::new().open(path).map_err(|_| ())?;
        let storage = env.create_database(Some("storage")).map_err(|_| ())?;
        let children = env.create_database(Some("children")).map_err(|_| ())?;
        Ok(HeedDb {
            env,
            db: Databases {
                storage,
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
            db: &self.db
        }
    }

    fn rw(&'db self) -> Self::RwTxn {
        Transaction {
            txn: self.env.write_txn().unwrap(),
            db: &self.db
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

impl<'db, T: Readable> CanRead for Transaction<'db, T> {
    fn get(&self, path: &[u8]) -> Vec<u8> {
        let data = self.db.storage.get(self.txn.readable(), path).unwrap();
        data.unwrap()
    }

    fn get_children(&self, path: &[u8]) -> ChildrenInfo {
        let data = self.db.children.get(self.txn.readable(), path).unwrap();
        data.unwrap()
    }
}

impl<'db> CanWrite for Transaction<'db, heed::RwTxn<'db>> {
    fn set(&mut self, path: &[u8], data: &[u8]) {
        self.db.storage.put(&mut self.txn, path, data).unwrap();
    }

    fn set_children(&mut self, parent: &[u8], children: &ChildrenInfo) {
        self.db.children.put(&mut self.txn, parent, children).unwrap();
    }

    fn remove(&mut self, path: &[u8]) {
        self.db.children.delete(&mut self.txn, path).unwrap();
    }
}
