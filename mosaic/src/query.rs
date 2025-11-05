use hi_sparse_bitset::{BitSetInterface, iter::IndexIter};

use crate::{
    BitSet, Index,
    store::{MaskStore, RawStore, Store},
};

pub trait Query {
    type Item;
    type Access;
    type Mask: BitSetInterface;

    fn open(self) -> (Self::Mask, Self::Access);

    unsafe fn get(access: &Self::Access, index: Index) -> Self::Item;
}

pub trait IntoQuery {
    type Item;
    type IntoQuery: Query<Item = Self::Item>;

    fn into_query(self) -> Self::IntoQuery;

    fn query(self) -> QueryIter<Self::IntoQuery>
    where
        Self: Sized,
    {
        QueryIter::new(self.into_query())
    }

    fn maybe(self) -> MaybeQuery<Self::IntoQuery>
    where
        Self: Sized,
    {
        MaybeQuery(self.into_query())
    }
}

impl<Q: Query> IntoQuery for Q {
    type Item = Q::Item;
    type IntoQuery = Q;

    fn into_query(self) -> Self::IntoQuery {
        self
    }
}

pub struct QueryIter<Q: Query> {
    mask_iter: IndexIter<Q::Mask>,
    access: Q::Access,
}

impl<Q: Query> QueryIter<Q> {
    pub fn new(query: Q) -> Self {
        let (mask, access) = query.open();
        let mask_iter = mask.into_block_iter().into_indices();
        Self { mask_iter, access }
    }
}

impl<Q: Query> Iterator for QueryIter<Q> {
    type Item = Q::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.mask_iter
            .next()
            .map(|index| unsafe { Q::get(&self.access, index) })
    }

    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        self.mask_iter.for_each(|index| {
            let item = unsafe { Q::get(&self.access, index) };
            f(item)
        });
    }
}

pub struct MaybeQuery<Q: Query> {
    inner: Q,
    mask: BitSet,
}

impl<Q: Query> Query for MaybeQuery<Q> {
    type Item = Option<Q::Item>;
    type Access = (Q::Mask, Q::Access);
    type Mask = &'a BitSet where Q: for<'a> 'a;

    fn open(self) -> (Self::Mask, Self::Access) {
        let (mask, access) = self.0.open();
        (BitSetAll, (mask, access))
    }

    unsafe fn get((mask, access): &Self::Access, index: Index) -> Self::Item {
        // Aliasing requirements must be upheld by the caller, but we ensure that no invalid index
        // is passed to our inner `Query`.
        if mask.contains(index) {
            Some(unsafe { Q::get(access, index) })
        } else {
            None
        }
    }
}

pub struct QueryTuple<T>(T);

pub trait BitSetAnd {
    type Value: BitSetInterface;

    fn bitset_and(self) -> Self::Value;
}

impl<'a, S: RawStore> Query for &'a MaskStore<S> {
    type Item = &'a S::Item;
    type Access = &'a S;
    type Mask = &'a BitSet;

    fn open(self) -> (Self::Mask, Self::Access) {
        (self.mask(), self.inner())
    }

    unsafe fn get(access: &Self::Access, index: Index) -> Self::Item {
        unsafe { access.get(index) }
    }
}

impl<'a, S: RawStore> Query for &'a mut MaskStore<S> {
    type Item = &'a mut S::Item;
    type Access = &'a S;
    type Mask = &'a BitSet;

    fn open(self) -> (Self::Mask, Self::Access) {
        (self.mask(), self.inner())
    }

    unsafe fn get(access: &Self::Access, index: Index) -> Self::Item {
        unsafe { access.get_mut(index) }
    }
}

macro_rules! define_query {
    ($first:ident $(, $rest:ident)*) => {
        impl<$first, $($rest),*> Query for QueryTuple<($first, $($rest),*)>
        where
            $first: Query,
            $($rest: Query<Mask: hi_sparse_bitset::BitSetBase<Conf = <$first::Mask as hi_sparse_bitset::BitSetBase>::Conf>>,)*
        {
            type Item = ($first::Item, $($rest::Item),*);
            type Access = ($first::Access, $($rest::Access),*);
            type Mask = <(<$first as Query>::Mask, $(<$rest as Query>::Mask),*) as BitSetAnd>::Value;

            #[allow(non_snake_case)]
            fn open(self) -> (Self::Mask, Self::Access) {
                let ($first, $($rest),*) = self.0;
                let ($first, $($rest),*) = ($first.open(), $($rest.open()),*);

                let mask = ($first.0, $($rest.0),*).bitset_and();
                let access = ($first.1, $($rest.1),*);
                (mask, access)
            }

            #[allow(non_snake_case)]
            unsafe fn get(access: &Self::Access, index: Index) -> Self::Item {
                let ($first, $($rest),*) = access;
                unsafe { ($first::get($first, index), $($rest::get($rest, index)),*) }
            }
        }
    };
}

define_query! {A}
define_query! {A, B}
define_query! {A, B, C}
define_query! {A, B, C, D}

macro_rules! define_into_query {
    ($first:ident $(, $rest:ident)*) => {
        impl<$first, $($rest),*> IntoQuery for ($first, $($rest),*)
        where
            $first: IntoQuery,
            $($rest: IntoQuery<IntoQuery: Query<Mask: hi_sparse_bitset::BitSetBase<Conf = <<$first::IntoQuery as Query>::Mask as hi_sparse_bitset::BitSetBase>::Conf>>>,)*
        {
            type Item = ($first::Item, $($rest::Item),*);
            type IntoQuery = QueryTuple<(<$first as IntoQuery>::IntoQuery, $(<$rest as IntoQuery>::IntoQuery),*)>;

            #[allow(non_snake_case)]
            fn into_query(self) -> Self::IntoQuery {
                let ($first, $($rest),*) = self;
                QueryTuple(($first.into_query(), $($rest.into_query()),*))
            }
        }
    };
}

define_into_query! {A}
define_into_query! {A, B}
define_into_query! {A, B, C}
define_into_query! {A, B, C, D}

macro_rules! define_bitset_and {
    ($first:ident, $($rest:ident),+ $(,)?) => {
        impl<$first, $($rest),*> BitSetAnd for ($first, $($rest),*)
        where
            $first: hi_sparse_bitset::BitSetInterface,
            $($rest: hi_sparse_bitset::BitSetInterface<Conf = $first::Conf>),*
        {
            type Value = hi_sparse_bitset::Apply<hi_sparse_bitset::ops::And, $first, <($($rest,)*) as BitSetAnd>::Value>;

            #[allow(non_snake_case)]
            fn bitset_and(self) -> Self::Value {
                let ($first, $($rest),*) = self;
                hi_sparse_bitset::apply(hi_sparse_bitset::ops::And, $first, ($($rest,)*).bitset_and())
            }
        }
    };

    ($first:ident $(,)?) => {
        impl<$first> BitSetAnd for ($first,)
        where
            $first: hi_sparse_bitset::BitSetInterface,
        {
            type Value = $first;

            fn bitset_and(self) -> Self::Value {
                self.0
            }
        }
    };
}

define_bitset_and! {A}
define_bitset_and! {A, B}
define_bitset_and! {A, B, C}
define_bitset_and! {A, B, C, D}
