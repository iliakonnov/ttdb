pub trait Is<T> {}
impl<T> Is<T> for T {}
impl Is<!> for std::convert::Infallible {}

pub trait TestIs<Other> {
    type Res: Bool;
}
impl<T, U> TestIs<U> for T {
    default type Res = No;
}
impl<T, U> TestIs<U> for T where T: Is<U> {
    type Res = Yes;
}

// NOTE: Compiler cannot verify that Is<U> and IsNot<U> will never be both implemented for some T
pub trait IsNot<T> {}
impl<T, U> IsNot<U> for T where T: TestIs<U, Res=No> {}

pub trait Bool {
    type Not: Bool;
    fn is() -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct Yes;
impl Bool for Yes {
    type Not = No;
    fn is() -> bool { true }
}

#[derive(Clone, Copy, Debug)]
pub struct No;
impl Bool for No {
    type Not = Yes;
    fn is() -> bool { false }
}
