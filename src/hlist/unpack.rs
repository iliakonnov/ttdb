#![allow(clippy::type_complexity)]
use super::*;

// TODO: Move to build.rs
/*# python
types = []
for i in range(1, 16+2):
    impl = ''
    cons = 'Nil'
    res = ''
    for j, t in enumerate(types):
        impl += t + ', '
        res += '('*j + 'self' + '.1)'*j + '.0, '
    for t in reversed(types):
        cons = f'Cons<{t}, {cons}>'
    types.append(f'T{i}')
    print(f'''
impl<{impl}> Unpack for {cons} {{
    type Tuple = ({impl});
    fn unpack(self) -> Self::Tuple {{
        ({res})
    }}
}}''')
*/
impl<> Unpack for Nil {
    type Tuple = ();
    #[allow(clippy::unused_unit)]
    fn unpack(self) -> Self::Tuple {
        ()
    }
}

impl<T1, > Unpack for Cons<T1, Nil> {
    type Tuple = (T1, );
    fn unpack(self) -> Self::Tuple {
        (self.0, )
    }
}

impl<T1, T2, > Unpack for Cons<T1, Cons<T2, Nil>> {
    type Tuple = (T1, T2, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, )
    }
}

impl<T1, T2, T3, > Unpack for Cons<T1, Cons<T2, Cons<T3, Nil>>> {
    type Tuple = (T1, T2, T3, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, )
    }
}

impl<T1, T2, T3, T4, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Nil>>>> {
    type Tuple = (T1, T2, T3, T4, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Nil>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Nil>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Nil>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Nil>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Nil>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Nil>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Nil>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Cons<T12, Nil>>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, (((((((((((self.1).1).1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Cons<T12, Cons<T13, Nil>>>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, (((((((((((self.1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Cons<T12, Cons<T13, Cons<T14, Nil>>>>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, (((((((((((self.1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).0, (((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Cons<T12, Cons<T13, Cons<T14, Cons<T15, Nil>>>>>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, (((((((((((self.1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).0, (((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).1).0, )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, > Unpack for Cons<T1, Cons<T2, Cons<T3, Cons<T4, Cons<T5, Cons<T6, Cons<T7, Cons<T8, Cons<T9, Cons<T10, Cons<T11, Cons<T12, Cons<T13, Cons<T14, Cons<T15, Cons<T16, Nil>>>>>>>>>>>>>>>> {
    type Tuple = (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, );
    fn unpack(self) -> Self::Tuple {
        (self.0, (self.1).0, ((self.1).1).0, (((self.1).1).1).0, ((((self.1).1).1).1).0, (((((self.1).1).1).1).1).0, ((((((self.1).1).1).1).1).1).0, (((((((self.1).1).1).1).1).1).1).0, ((((((((self.1).1).1).1).1).1).1).1).0, (((((((((self.1).1).1).1).1).1).1).1).1).0, ((((((((((self.1).1).1).1).1).1).1).1).1).1).0, (((((((((((self.1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).0, (((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).0, ((((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).1).0, (((((((((((((((self.1).1).1).1).1).1).1).1).1).1).1).1).1).1).1).0, )
    }
}
