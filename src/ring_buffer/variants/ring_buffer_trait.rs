use crate::ring_buffer::storage::Storage;

/// Trait implemented by concurrent ring buffer.
pub trait ConcurrentRB {}

/// Trait implemented by ring buffers.
#[allow(private_bounds)]
pub trait MutRB:
    PrivateIterManager + IterManager + StorageManager<StoredType = Self::Item>
{
    type Item;
}

pub(crate) trait PrivateIterManager {
    fn set_alive_iters(&self, count: u8);
    fn drop_iter(&self) -> u8;
    fn acquire_fence(&self);
}

/// Trait used to manage indices.
pub trait IterManager {
    fn prod_index(&self) -> usize;
    fn work_index(&self) -> usize;
    fn cons_index(&self) -> usize;
    fn set_prod_index(&self, index: usize);
    fn set_work_index(&self, index: usize);
    fn set_cons_index(&self, index: usize);
    fn alive_iters(&self) -> u8;
}

/// Trait used to manage storage.
pub(crate) trait StorageManager {
    type StoredType;
    type S: Storage<Item = Self::StoredType>;

    fn inner(&self) -> &Self::S;
    #[allow(clippy::mut_from_ref)]
    fn inner_mut(&self) -> &mut Self::S;
    fn inner_len(&self) -> usize;
}
