use crate::hlist::*;
use crate::versions::*;
use crate::path::*;

trait Path {}

trait Database {
    type RoTxn: RoTransaction;
    type RwTxn: RoTransaction + RwTransaction;
    fn ro(&self) -> Self::RoTxn;
    fn rw(&self) -> Self::RwTxn;
}

trait AnyTransaction: Sized {}

trait RwTransaction: AnyTransaction {}

trait RoAccess {
    fn get<V, P: Path>(&self, _path: &P) -> V;

    fn children<P: Path>(&self, _path: &P) -> ChildrenInfo;
}

trait RoTransaction: AnyTransaction + RoAccess {
    fn get<P: Path>(self, path: &P) -> NodeRef<Self, P, Nil> {
        NodeRef {
            txn: self,
            path,
            out: Nil
        }
    }
}

struct NodeRef<'a, Txn, Path, Out> {
    txn: Txn,
    path: &'a Path,
    out: Out
}

impl<'a, T, P, O> NodeRef<'a, T, P, O> {
    pub fn split(self) -> (T, O) {
        (self.txn, self.out)
    }

    pub fn done(self) -> (T, O::Tuple) where O: Unpack {
        (self.txn, self.out.unpack())
    }

    pub fn get<V>(self) -> NodeRef<'a, T, P, Cons<V, O>> where
        V: Version,
        O: HList,
        T: RoTransaction,
        P: Path
    {
        let val = <T as RoAccess>::get(&self.txn, self.path);
        let out = self.out.push(val);
        NodeRef {
            out,
            txn: self.txn,
            path: self.path
        }
    }

    pub fn children(self) -> NodeRef<'a, T, P, Cons<ChildrenInfo, O>> where
        O: HList,
        T: RoTransaction,
        P: Path
    {
        let val = <T as RoAccess>::children(&self.txn, self.path);
        let out = self.out.push(val);
        NodeRef {
            out,
            txn: self.txn,
            path: self.path
        }
    }
}
