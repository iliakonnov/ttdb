// Stolen from fntools: https://github.com/WaffleLapkin/fntools/blob/da2ef6c881c50ed95bfb28b42330e9e13df9461a/src/local_macros.rs#L9
macro_rules! for_tuples {
    ( $( $types:ident, )* @ # $cb:ident) => {
        $cb!($( $types, )*);
    };
    ( $( $types:ident, )* @ $ty:ident, $( $rest:ident, )* # $cb:ident) => {
        $cb!($( $types, )*);
        for_tuples!($( $types, )* $ty, @ $( $rest, )* # $cb);
    };
    ( $ty:ident, $( $rest:ident, )* # $cb:ident) => {
        for_tuples!( $ty, @ $( $rest, )* # $cb);
    };
    () => {};
}
