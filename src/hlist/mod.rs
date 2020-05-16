use std::mem::MaybeUninit;
use fntools::tuple::append::TupleAppend;
use fntools::for_tuples;

pub trait HList: Sized {
    type Item;
    type Rest: HList;
    fn push<T>(self, val: T) -> Cons<T, Self> {
        Cons(val, self)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Cons<T, L>(pub T, pub L);

#[derive(Debug, Copy, Clone)]
pub struct Nil;

pub trait NonNil: HList {}
impl !NonNil for Nil {}
impl<T, L: NonNil> NonNil for Cons<T, L> {}

impl HList for Nil {
    type Item = Nothing;
    type Rest = Nil;
}
impl<T, L: HList> HList for Cons<T, L> {
    type Item = T;
    type Rest = L;
}

pub type Nothing = !;
pub trait IsItem<T> {}

pub auto trait SomeItem {}
impl !SomeItem for Nothing {}

impl<T: SomeItem> IsItem<T> for T {}
impl<T: SomeItem> IsItem<T> for Nothing {}

pub trait Unpack {
    type Tuple;
    fn unpack(self) -> Self::Tuple;
}

impl Unpack for Nil {
    type Tuple = ();
    #[allow(clippy::unused_unit)]
    fn unpack(self) -> Self::Tuple {
        ()
    }
}

// TODO: Probably Unpack implementation can be improved by removing macro usage at all.
// But current is good enough
#[macro_export]
#[allow(non_snake_case)]
macro_rules! HList {
    [] => {$crate::hlist::Nil};
    [$t:ty $(, $ts:ty)* $(,)?] => {$crate::hlist::Cons<$t, $crate::HList![$($ts),*]>};
}

macro_rules! unpack_impl {
    ($($t:ident,)*) => {
        impl<$($t),*> Unpack for HList![$($t),*] {
            type Tuple = ($($t, )*);
            fn unpack(self) -> Self::Tuple {
                let rest = self.1.unpack();
                rest.append(self.0)
            }
        }
    };
}

fntools::for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, # unpack_impl);

pub trait SplitRes: HList {
    type Good: HList;
    type Fail: HList;
    fn split_res(self) -> (Self::Good, Self::Fail);
}

impl SplitRes for Nil {
    type Good = Self;
    type Fail = Self;
    fn split_res(self) -> (Self::Good, Self::Fail) {
        (Self, Self)
    }
}

impl<T, E, L: SplitRes> SplitRes for Cons<Result<T, E>, L> {
    type Good = Cons<Option<T>, L::Good>;
    type Fail = Cons<Option<E>, L::Fail>;
    fn split_res(self) -> (Self::Good, Self::Fail) {
        let (g, f) = match self.0 {
            Ok(g) => (Some(g), None),
            Err(f) => (None, Some(f)),
        };
        let (gs, fs) = self.1.split_res();
        (Cons(g, gs), Cons(f, fs))
    }
}

#[macro_export]
macro_rules! map_hlist {
    {
        $vis:vis trait $tr:ident |$arg:ident : $typ:ident| -> ($($ret:tt)*) where ($($bound:tt)*) {$($body:tt)*}
    } => {
        $vis trait $tr where
            Self::Out: $crate::hlist::Homogenous<Value=$($ret)*>,
        {
            type Out;
            fn do_map_hlist(self) -> Self::Out;
        }
        impl<$typ, L: $tr> $tr for $crate::hlist::Cons<$typ, L> where $($bound)* {
            type Out = $crate::hlist::Cons<$($ret)*, <L as $tr>::Out>;
            fn do_map_hlist(self) -> Self::Out {
                #[allow(unused_mut)] let mut $arg: $typ = self.0;
                let res: $($ret)* = {
                    $($body)*
                };
                let rest = <L as $tr>::do_map_hlist(self.1);
                $crate::hlist::Cons(res, rest)
            }
        }
        impl<$typ> $tr for $crate::hlist::Cons<$typ, Nil> where $($bound)* {
            type Out = $crate::hlist::Cons<$($ret)*, Nil>;
            fn do_map_hlist(self) -> Self::Out {
                #[allow(unused_mut)] let mut $arg: $typ = self.0;
                let res: $($ret)* = {
                    $($body)*
                };
                $crate::hlist::Cons(res, Nil)
            }
        }
    };

    ($list:expr => $($t:tt)*) => {
        {
            $crate::map_hlist! {
                trait HListMapHelper $($t)*
            }
            $list.do_map_hlist()
        }
    };
}

#[macro_export]
macro_rules! foldl_hlist {
    {
        $vis:vis trait $tr:ident |$start:ident, $arg:ident : $typ:ident| -> ($($ret:tt)*) where ($($bound:tt)*) {$($body:tt)*}
    } => {
        $vis trait $tr {
            fn do_foldl_hlist(self, $start: $($ret)*) -> $($ret)*;
        }
        impl<$typ, L: $tr> $tr for $crate::hlist::Cons<$typ, L> where $($bound)* {
            fn do_foldl_hlist(self, #[allow(unused_mut)] mut $start: $($ret)*) -> $($ret)* {
                #[allow(unused_mut)] let mut $arg = self.0;
                let new: $($ret)* = {
                    $($body)*
                };
                <L as $tr>::do_foldl_hlist(self.1, new)
            }
        }
        impl<$typ> $tr for $crate::hlist::Cons<$typ, $crate::hlist::Nil> where $($bound)* {
            fn do_foldl_hlist(self, #[allow(unused_mut)] mut $start: $($ret)*) -> $($ret)* {
                #[allow(unused_mut)] let mut $arg = self.0;
                {
                    $($body)*
                }
            }
        }
    };

    ($list:expr => $start:expr => $($t:tt)*) => {
        {
            $crate::foldl_hlist! {
                trait HListFoldlHelper $($t)*
            }
            $list.do_foldl_hlist($start)
        }
    };
}

// I like the idea, but this code does not compiling:
//     error[E0275]: overflow evaluating the requirement `hlist::Nil: hlist::CreateHList<V, {N-1}>`
// Increasing #![recursion_limit] does not help, so I just left it commented.
// TODO: One time const generics will be ready...
/*
pub trait CreateHList<Value, const LENGTH: u8> {
    type Out;
}

impl<V> CreateHList<V, 0> for Nil {
    type Out = Nil;
}

impl<V, const N: u8> CreateHList<V, N> for Nil {
    type Out = Cons<V, <Nil as CreateHList<V, {N-1}>>::Out>;
}
*/

pub trait Homogenous: HList {
    type Value;
}

impl<T> Homogenous for Cons<T, Nil> {
    type Value = T;
}

impl<T, L> Homogenous for Cons<T, L> where
    L: Homogenous<Value=T>,
{
    type Value = L::Value;
}

pub trait Length: HList {
    const LENGTH: usize;

    fn length(&self) -> usize {
        Self::LENGTH
    }
}

impl Length for Nil {
    const LENGTH: usize = 0;
}

impl<T, L: Length> Length for Cons<T, L> {
    const LENGTH: usize = 1 + L::LENGTH;
}

pub trait ToSlice: Homogenous+Length {
    /// Panics if `slice.len() < Self::LENGTH`
    fn init_slice(self, slice: &mut [MaybeUninit<Self::Value>]);

    /// Panics if `LEN != Self::LENGTH`
    fn into_array<const LEN: usize>(self) -> [Self::Value; LEN]
    {
        assert_eq!(LEN, Self::LENGTH);
        let mut data: [MaybeUninit<Self::Value>; LEN] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        self.init_slice(&mut data);
        let data: [Self::Value; LEN] = unsafe {
            #[allow(trivial_casts)]
            (&data as *const _ as *const [Self::Value; LEN]).read()
        };
        data
    }

    /// Guarantees that slice size equals to `Self::LENGTH`
    fn into_slice(self) -> Box<[Self::Value]> {
        let mut slice = Box::new_uninit_slice(Self::LENGTH);
        self.init_slice(&mut slice);
        unsafe { slice.assume_init() }
    }

    /// Guarantees that vector size equals to `Self::LENGTH`
    fn into_vec(self) -> Vec<Self::Value>
    {
        self.into_slice().into_vec()
    }
}

impl<T> ToSlice for Cons<T, Nil> where Self: Length + Homogenous<Value=T> {
    fn init_slice(self, slice: &mut [MaybeUninit<Self::Value>]) {
        slice[0].write(self.0);
    }
}

impl<T, L> ToSlice for Cons<T, L> where
    Self: Length + Homogenous<Value=T>,
    L: ToSlice + Homogenous<Value=T>
{
    fn init_slice(self, slice: &mut [MaybeUninit<Self::Value>]) {
        slice[0].write(self.0);
        self.1.init_slice(&mut slice[1..]);
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_all!(Nil: Homogenous);
    sa::assert_impl_all!(Cons<i32, Nil>: Homogenous);
    sa::assert_impl_all!(Cons<i32, Cons<i32, Nil>>: Homogenous);
    sa::assert_type_eq_all!(
        i32,
        <Cons<i32, Nil> as Homogenous>::Value,
        <Cons<i32, Cons<i32, Nil>> as Homogenous>::Value,
        <Cons<i32, Cons<i32, Cons<i32, Nil>>> as Homogenous>::Value,
    );
    sa::assert_impl_all!(<Cons<i32, Cons<i32, Cons<i32, Nil>>> as Homogenous>::Value: IsItem<i32>);
    sa::assert_type_eq_all!(HList![], Nil);
    sa::assert_type_eq_all!(HList![i32], Cons<i32, Nil>);
    sa::assert_type_eq_all!(HList![i32, u8], Cons<i32, Cons<u8, Nil>>);
    sa::assert_impl_all!(<Cons<i32, Cons<i32, Nil>> as HList>::Item: IsItem<i32>);
    sa::assert_impl_all!(<Nil as HList>::Item: IsItem<i32>);

    #[test]
    fn unpack() {
        let list = Cons(1, Cons('a', Cons("abc", Nil)));
        let tup = list.unpack();
        assert_eq!(tup, (1, 'a', "abc"));
    }

    #[test]
    fn push() {
        let list: Cons<u32, Nil> = Cons(1, Nil);
        let list: Cons<char, Cons<u32, Nil>> = list.push('a');
        assert_eq!(list.0, 'a');
        assert_eq!((list.1).0, 1);
    }

    #[test]
    fn split() {
        type R = Result<u8, u8>;
        let list = Cons(R::Err(1), Cons(R::Ok(2), Cons(R::Err(3), Nil)));
        let (good, fail) = list.split_res();
        assert_eq!(good.unpack(), (None, Some(2), None));
        assert_eq!(fail.unpack(), (Some(1), None, Some(3)));
    }

    #[test]
    fn length() {
        let list = Cons('a', Cons('b', Cons('c', Nil)));
        assert_eq!(list.length(), 3);
    }

    #[test]
    fn to_vec() {
        let list = Cons('a', Cons('b', Cons('c', Nil)));
        let vec = list.into_vec();
        assert_eq!(vec, vec!['a', 'b', 'c']);
    }

    #[test]
    fn to_slice() {
        let list = Cons('a', Cons('b', Cons('c', Nil)));
        let slice = list.into_slice();
        assert_eq!(&slice[..], &['a', 'b', 'c']);
    }

    #[test]
    fn to_array() {
        let list = Cons('a', Cons('b', Cons('c', Nil)));
        let arr = list.into_array();
        assert_eq!(arr, ['a', 'b', 'c']);
    }

    #[test]
    #[should_panic]
    fn to_wrong_array() {
        let list = Cons('a', Cons('b', Cons('c', Nil)));
        // List have length 3, but we are converting it into array of length 1.
        let _arr: [char; 1] = list.into_array();
    }

    #[test]
    fn map() {
        let list = Cons(1, Cons('a', Cons("abc", Nil)));
        let mapped = map_hlist!(list => |i: Foo| -> (String) where (Foo: std::fmt::Display) {
            format!("{}", i)
        });
        let mapped: HList![String, String, String] = mapped;
        let slice = mapped.into_slice();
        assert_eq!(&slice[..], &["1", "a", "abc"]);
    }

    #[test]
    fn sum() {
        let list = Cons(1_u8, Cons(-2_i16, Cons(3_u32, Nil)));
        let folded: i64 = foldl_hlist!(list => 0 => |a, b: T| -> (i64) where (T: Into<i64>) {
            a + b.into()
        });
        assert_eq!(folded, 1-2+3);
    }
}
