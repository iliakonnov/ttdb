/// Базовый трейт. Этот трейт реализуют все элементы цепочки версий.
/// Если тип хочет его реализовать, то он должен написать,какой тип будет первой версией.
/// По сути он означает, что тип принадлежит определенной цепочки версий.
pub trait Version: Sized
where
    // Очевидно, первая версия цепочки тоже должна относиться к этой цепочке
    // Поэтому требуем, чтобы она реализовывала Version с таким же FirstVersion
    Self::FirstVersion: Version<FirstVersion=Self::FirstVersion>,
    // И требуем, чтобы первая версия была обязательно отмечена как первая.
    // Это накладывает на неё необходимые ограничения, ибо не всякая версия может быть первой
    Self::FirstVersion: FirstVersion,
    // Обязательно можно узнать номер текущей версии
    Self: Counter
{
    type FirstVersion;
}

/// Трейт, который означает, что этот элемент цепочки не последний.
/// Позволяет узнать, какой элемент будет следующим
pub trait NextVersionRef: Version
where
    // Следующая версия должна относиться к той же цепочке
    Self::NextVersion: Version<FirstVersion=Self::FirstVersion>,
    // И причем след. версия должна указать в кач-ве предыдущей именно этот тип
    Self::NextVersion: PrevVersionRef<PrevVersion=Self>
{
    type NextVersion;
}

/// То же, что и NextVersionRef, но для предыдущей версии
pub trait PrevVersionRef
where
    // Этот тип должен относиться к какой-нибудь цепочке
    Self: Version,
    // А предыдущая версия должна относиться к той же цепочке, что и этот тип
    Self::PrevVersion: Version<FirstVersion=Self::FirstVersion>
{
    type PrevVersion;
}

/// Пометка, что данная версия является первой в цепочке
pub auto trait FirstVersion
where
    // Она первая тогда, когда в цепочке указано, что цепочка начинается с этой версии
    Self: Version<FirstVersion=Self>,
{
}

// И сразу требуем, чтобы первые версии не могли иметь предыдущих версий
impl<T> !PrevVersionRef for T where T: FirstVersion {}

/// Пометка, что данная версия является последней в цепочке
pub auto trait LastVersion where Self: PrevVersionRef {}
//impl<T> !NextVersionRef for T where T: LastVersion {}
// К сожалению, Rust на данный момент не позволяет автоматически реализовать LastVersion для всех подходящих типов

/// Макрос для описания цепочек, который реализует все необходимые трейты (по три на каждую версию)
#[macro_export]
macro_rules! chain {
    (
        $v1:ty $(=> $other:ty)*
    ) => {
        $crate::chain!([$v1] $v1 $(=> $other)*);
    };
    (
        [$first:ty] $v1:ty => $v2:ty $(=> $other:ty)*
    ) => {
        impl $crate::versions::Version for $v1 {
            type FirstVersion = $first;
        }
        impl $crate::versions::NextVersionRef for $v1 {
            type NextVersion = $v2;
        }
        impl $crate::versions::PrevVersionRef for $v2 {
            type PrevVersion = $v1;
        }

        $crate::chain!([$first] $v2 $(=> $other)*);
    };
    ( [$first:ty] $last:ty ) => {
        impl $crate::versions::LastVersion for $last {
        }
        impl $crate::versions::Version for $last {
            type FirstVersion = $first;
        }
    };
}

pub trait LastVersionRef where
    Self: Version,
    // Последняя версия обязательно должна находиться в той же цепочке, что и эта версия
    Self::LastVersion: Version<FirstVersion=Self::FirstVersion>,
    Self::LastVersion: LastVersion,
{
    type LastVersion;
}

// Если тип не реализует NextVersionRef, то он последний в цепочке
impl<T> LastVersionRef for T where T: Version,
{
    default type LastVersion = T;
}

// Если же тип не последний, то используется эта реализация
impl<T> LastVersionRef for T where T: Version + NextVersionRef,
{
    type LastVersion = <T::NextVersion as LastVersionRef>::LastVersion;
}

/// Трейт, при помощи которого будет реализован подсчет.
/// Его нельзя реализовывать вручную, поскольку это может нарушить гарантии:
/// - FirstVersion всегда имеет номер 0
/// - Все последующие версии отличаются ровно на единицу
pub trait Counter {
    const VERSION: usize;
    const TYPE_NAME: &'static str;
}

// Аналогично приёму с LastVersionRef, по-умолчанию версия нулевая.
impl<T> Counter for T where T: Version {
    default const VERSION: usize = 0;
    default const TYPE_NAME: &'static str = std::any::type_name::<T>();
}

// У всех последующих версий они будут отличаться ровно на единицу
impl<T> Counter for T where
    T: Version + PrevVersionRef,
    <T as PrevVersionRef>::PrevVersion: Counter
{
    const VERSION: usize = 1 + <T as PrevVersionRef>::PrevVersion::VERSION;
    const TYPE_NAME: &'static str = std::any::type_name::<T>();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VersionNotFound;

// Отдельный модуль, поскольку вспомогательные трейты должны там и остаться
mod loader {
    use super::{Version, NextVersionRef, PrevVersionRef, LastVersionRef, Counter};
    use std::error::Error;
    use std::boxed::Box;

    pub trait Serde: Version {
        fn save(self) -> Result<Vec<u8>, Box<dyn Error>>;
        fn load(data: Vec<u8>) -> Result<Self, Box<dyn Error>>;
    }

    pub trait Upgradeable: PrevVersionRef {
        fn upgrade(prev: Self::PrevVersion) -> Result<Self, Box<dyn Error>>;
    }

    pub trait Downgradeable: PrevVersionRef {
        fn downgrade(self) -> Result<Self::PrevVersion, Box<dyn Error>>;
    }

    #[derive(Debug)]
    pub enum LoadError {
        VersionTooBig {
            version: usize,
            max: usize
        },
        Migration(Box<dyn Error>),
        Load(Box<dyn Error>),
        NoMigration {
            from_version: usize,
            to_version: usize,
            to_name: &'static str
        }
    }

    pub fn load<T: Serde>(version: usize, data: Vec<u8>) -> Result<T, LoadError> {
        if version > <T as LastVersionRef>::LastVersion::VERSION {
            return Err(LoadError::VersionTooBig {
                version,
                max: <T as LastVersionRef>::LastVersion::VERSION
            })
        }
        if version < T::VERSION {
            // В данных слишком маленькая версия, нужно сначала загрузить предыдущую версию
            T::load_prev(version, data)
                .map_err(|e| fill_migration(e, version, T::VERSION, T::TYPE_NAME))
        } else {
            T::load_next(version, data)
                .map_err(|e| fill_migration(e, version, T::VERSION, T::TYPE_NAME))
        }
    }

    trait Fallback: Serde {
        fn load_fallback(version: usize, data: Vec<u8>) -> Result<Self, LoadError>;
    }

    impl<T> Fallback for T where T: Serde {
        fn load_fallback(version: usize, data: Vec<u8>) -> Result<Self, LoadError> {
            if version == Self::VERSION {
                Self::load(data).map_err(LoadError::Load)
            } else {
                // Will be filled later
                Err(LoadError::NoMigration {
                    from_version: 0,
                    to_version: 0,
                    to_name: ""
                })
            }
        }
    }

    fn fill_migration(e: LoadError, from_version: usize, to_version: usize, to_name: &'static str) -> LoadError {
        match e {
            LoadError::NoMigration {
                from_version: 0,
                to_version: 0,
                to_name: "",
            } => LoadError::NoMigration {
                from_version,
                to_name,
                to_version
            },
            x => x
        }
    }

    trait Prev: Serde {
        fn load_prev(version: usize, data: Vec<u8>) -> Result<Self, LoadError>;
    }

    impl<T> Prev for T where T: Fallback {
        default fn load_prev(version: usize, data: Vec<u8>) -> Result<Self, LoadError> {
            T::load_fallback(version, data)
        }
    }

    impl<T> Prev for T where
        T: Serde + Upgradeable + PrevVersionRef,
        T::PrevVersion: Serde,
        T::PrevVersion: NextVersionRef<NextVersion=Self>
    {
        fn load_prev(version: usize, data: Vec<u8>) -> Result<Self, LoadError> {
            if version == Self::VERSION {
                Self::load(data).map_err(LoadError::Load)
            } else {
                assert!(version < Self::VERSION);
                let prev = T::PrevVersion::load_prev(version, data)
                    .map_err(|e| fill_migration(e, version, Self::VERSION, Self::TYPE_NAME))?;
                T::upgrade(prev).map_err(LoadError::Migration)
            }
        }
    }
    trait Next: Serde {
        fn load_next(version: usize, data: Vec<u8>) -> Result<Self, LoadError>;
    }

    impl<T> Next for T where T: Fallback {
        default fn load_next(version: usize, data: Vec<u8>) -> Result<Self, LoadError> {
            T::load_fallback(version, data)
        }
    }
    impl<T> Next for T where
        T: Serde + NextVersionRef,
        T::NextVersion: Serde + Downgradeable,
        T::NextVersion: PrevVersionRef<PrevVersion=Self>
    {
        fn load_next(version: usize, data: Vec<u8>) -> Result<Self, LoadError> {
            if version == Self::VERSION {
                Self::load(data).map_err(LoadError::Load)
            } else {
                assert!(version > Self::VERSION);
                let next = T::NextVersion::load_next(version, data)
                    .map_err(|e| fill_migration(e, version, Self::VERSION, Self::TYPE_NAME))?;
                T::NextVersion::downgrade(next).map_err(LoadError::Migration)
            }
        }
    }
}

pub use loader::*;

mod serde {
    use serde::{Deserialize, Serialize};
    use super::{Serde, Version};
    use std::error::Error;

    impl<T> Serde for T where
        T: for<'de> Deserialize<'de> + Serialize,
        T: Version
    {
        fn save(self) -> Result<Vec<u8>, Box<dyn Error>> {
            let val = rmpv::ext::to_value(self)?;
            let mut buf = Vec::new();
            rmpv::encode::write_value(&mut buf, &val)?;
            Ok(buf)
        }

        fn load(data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
            let mut cur = std::io::Cursor::new(data);
            let val = rmpv::decode::read_value(&mut cur)?;
            let res = rmpv::ext::from_value(val)?;
            Ok(res)
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::blacklisted_name)]
    extern crate static_assertions as sa;
    use super::*;

    // Подготовим типы
    #[derive(Debug, Eq, PartialEq)] struct Foo;
    #[derive(Debug, Eq, PartialEq)] struct Bar;
    #[derive(Debug, Eq, PartialEq)] struct Baz;
    chain!(Foo => Bar => Baz);

    sa::const_assert_eq!(Foo::VERSION, 0);
    sa::const_assert_eq!(Bar::VERSION, 1);
    sa::const_assert_eq!(Baz::VERSION, 2);

    // Проверяем, что автоматически реализовался FirstVersion
    sa::assert_impl_all!(Foo: FirstVersion);

    sa::assert_type_eq_all!(<Foo as NextVersionRef>::NextVersion, Bar);  // Foo -> Bar
    sa::assert_type_eq_all!(<Bar as PrevVersionRef>::PrevVersion, Foo);  // Bar <- Foo
    sa::assert_type_eq_all!(<Bar as NextVersionRef>::NextVersion, Baz);  // Bar -> Baz
    sa::assert_type_eq_all!(<Baz as PrevVersionRef>::PrevVersion, Bar);  // Baz <- Bar

    // Проверяем, что все в одной конкретной цепочке
    sa::assert_type_eq_all!(
        Foo,
        <Foo as Version>::FirstVersion,
        <Bar as Version>::FirstVersion,
        <Baz as Version>::FirstVersion,
    );

    sa::assert_type_eq_all!(
        // С точки зрения компилятора LastVersion != Last, потому что это нигде явно не гарантируется
        // Но на самом деле по-другому быть не может
        <Foo as LastVersionRef>::LastVersion,
        <Bar as LastVersionRef>::LastVersion,
        <Baz as LastVersionRef>::LastVersion,
    );
    sa::const_assert_eq!(<Foo as LastVersionRef>::LastVersion::VERSION, Baz::VERSION);

    // Утомительная реализация сериализации и десериализации...
    use std::error::Error;

    impl Serde for Foo {
        fn save(self) -> Result<Vec<u8>, Box<dyn Error>> { Ok(vec![1]) }

        fn load(mut data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
            match data.pop() {
                Some(1) => Ok(Foo),
                x => Err(format!("Error loading Foo: {:?}", x).into()),
            }
        }
    }

    impl Serde for Bar {
        fn save(self) -> Result<Vec<u8>, Box<dyn Error>> { Ok(vec![2]) }

        fn load(mut data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
            match data.pop() {
                Some(2) => Ok(Bar),
                x => Err(format!("Error loading Bar: {:?}", x).into()),
            }
        }
    }

    impl Serde for Baz {
        fn save(self) -> Result<Vec<u8>, Box<dyn Error>> { Ok(vec![3]) }

        fn load(mut data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
            match data.pop() {
                Some(3) => Ok(Baz),
                x => Err(format!("Error loading Baz: {:?}", x).into()),
            }
        }
    }

    // Foo -> Bar
    impl Upgradeable for Bar {
        fn upgrade(_prev: Foo) -> Result<Self, Box<dyn Error>> {
            Ok(Bar)
        }
    }

    // Bar <-> Baz
    impl Downgradeable for Baz {
        fn downgrade(self) -> Result<Bar, Box<dyn Error>> {
            Ok(Bar)
        }
    }
    impl Upgradeable for Baz {
        fn upgrade(_prev: Bar) -> Result<Self, Box<dyn Error>> {
            Ok(Baz)
        }
    }

    #[test]
    fn migrations() {
        // Upgrading
        let (version, data) = (Foo::VERSION, Foo.save().unwrap());
        let bar = load::<Bar>(version, data).unwrap();

        // Upgrade is possible: Foo -> Bar
        let (version, data) = (Bar::VERSION, bar.save().unwrap());
        let baz = load::<Baz>(version, data).unwrap();

        // Downgrading
        let (version, data) = (Baz::VERSION, baz.save().unwrap());
        let bar = load::<Bar>(version, data).unwrap();

        // No downgrade is possible: Foo <\- Bar
        let (version, data) = (Bar::VERSION, bar.save().unwrap());
        let foo = load::<Foo>(version, data);
        assert!(matches!(foo,
            Err(LoadError::NoMigration {
                from_version: 1,  // From Bar
                to_version: 0, // to Foo
                to_name: "fstreedb::versions::test::Foo"
            })
        ));
    }
}
