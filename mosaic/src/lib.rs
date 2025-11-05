mod entity;
mod query;
mod store;

pub use mosaic_derive::Mosaic;

pub use self::entity::*;
pub use self::query::*;
pub use self::store::*;

pub type BitSet = hi_sparse_bitset::BitSet<hi_sparse_bitset::config::_128bit>;
pub type Index = usize;
pub type Handle = u64;
