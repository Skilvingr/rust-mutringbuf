use crate::ring_buffer::storage::storage_trait::Storage;

/// Trait implemented by local ring buffer.
pub trait LocalRB {}
/// Trait implemented by concurrent ring buffer.
pub trait ConcurrentRB {}
/// Trait implemented by ring buffers.
#[allow(private_bounds)]
pub trait MutRB<T>: IterManager + StorageManager<StoredType = T> {}

/// Trait used to manage indices
pub trait IterManager {
    fn prod_index(&self) -> usize;
    fn work_index(&self) -> usize;
    fn cons_index(&self) -> usize;
    fn set_prod_index(&self, index: usize);
    fn set_work_index(&self, index: usize);
    fn set_cons_index(&self, index: usize);

    fn prod_alive(&self) -> bool;
    fn work_alive(&self) -> bool;
    fn cons_alive(&self) -> bool;
    fn set_prod_alive(&self, alive: bool);
    fn set_work_alive(&self, alive: bool);
    fn set_cons_alive(&self, alive: bool);
}

/// Trait used to manage storage.
pub(crate) trait StorageManager {
    type StoredType;
    type S: Storage<Self::StoredType>;

    fn inner(&self) -> &Self::S;
    fn inner_mut(&mut self) -> &mut Self::S;
    fn inner_len(&self) -> usize;
}