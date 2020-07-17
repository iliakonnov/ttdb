use std::mem::MaybeUninit;
use fntools::tuple::append::TupleAppend;

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
    type Item = !;
    type Rest = Nil;
}
impl<T, L: HList> HList for Cons<T, L> {
    type Item = T;
    type Rest = L;
}

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

#[macro_export]
#[allow(non_snake_case)]
macro_rules! hlist {
    [] => {$crate::hlist::Nil};
    [$x:expr $(, $xs:expr)* $(,)?] => {$crate::hlist::Cons($x, $crate::hlist![$($xs),*])};
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

for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, # unpack_impl);

pub trait UnwrapAll: HList {
    type Good: HList;
    fn unwrap_all(self) -> Self::Good;
}

impl UnwrapAll for Nil {
    type Good = Nil;
    fn unwrap_all(self) -> Self::Good { Nil }
}

impl<T, E: std::fmt::Debug, L: UnwrapAll> UnwrapAll for Cons<Result<T, E>, L> {
    type Good = Cons<T, L::Good>;
    fn unwrap_all(self) -> Self::Good {
        let unwrapped = self.0.unwrap();
        let rem = self.1.unwrap_all();
        Cons(unwrapped, rem)
    }
}

impl<T: UnwrapAll, L: UnwrapAll> UnwrapAll for Cons<T, L> {
    type Good = Cons<T::Good, L::Good>;
    fn unwrap_all(self) -> Self::Good {
        let unwrapped = self.0.unwrap_all();
        let rem = self.1.unwrap_all();
        Cons(unwrapped, rem)
    }
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

pub trait Append<T>: HList {
    type Result: HList;
    fn append(self, elem: T) -> Self::Result;
}

impl<T> Append<T> for Nil {
    type Result = Cons<T, Nil>;
    fn append(self, elem: T) -> Self::Result {
        Cons(elem, Nil)
    }
}

impl<T, U, L: Append<T>> Append<T> for Cons<U, L> {
    type Result = Cons<U, <L as Append<T>>::Result>;
    fn append(self, elem: T) -> Self::Result {
        Cons(self.0, self.1.append(elem))
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use std::mem::drop;
    use super::*;

    sa::assert_not_impl_any!(Nil: Homogenous);
    sa::assert_impl_all!(Cons<i32, Nil>: Homogenous);
    sa::assert_impl_all!(Cons<i32, Cons<i32, Nil>>: Homogenous);
    sa::assert_type_eq_all!(
        i32,
        <Cons<i32, Nil> as Homogenous>::Value,
        <Cons<i32, Cons<i32, Nil>> as Homogenous>::Value,
        <Cons<i32, Cons<i32, Cons<i32, Nil>>> as Homogenous>::Value,
    );
    sa::assert_type_eq_all!(HList![], Nil);
    sa::assert_type_eq_all!(HList![i32], Cons<i32, Nil>);
    sa::assert_type_eq_all!(HList![i32, u8], Cons<i32, Cons<u8, Nil>>);

    sa::assert_type_eq_all!(<HList![i32, u8] as Append<char>>::Result, HList![i32, u8, char]);

    #[test]
    fn unpack() {
        let list = Cons(1, Cons('a', Cons("abc", Nil)));
        let tup = list.unpack();
        assert_eq!(tup, (1, 'a', "abc"));
    }

    #[test]
    fn hlist_macro() {
        let list: HList![u32, char, &'static str] = hlist![1, 'a', "abc"];
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
    fn unwrap_all() {
        let list: HList![Result<i32, ()>, Result<char, ()>] = hlist![Ok(1), Ok('a')];
        let unwrapped: (i32, char) = list.unwrap_all().unpack();
        assert_eq!(unwrapped, (1, 'a'));
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
    fn clone() {
        let list = Cons(1, Cons('a', Cons("abc".to_string(), Nil)));
        let cloned = list.clone();
        assert_eq!(cloned.unpack(), (1, 'a', "abc".to_string()));
        drop(list);
    }

    #[test]
    fn copy() {
        let list = Cons(1, Cons('a', Nil));
        let copied = list;
        assert_eq!(copied.unpack(), (1, 'a'));
        #[allow(clippy::drop_copy)] drop(list);
    }
}
