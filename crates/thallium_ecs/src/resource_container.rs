use crate::Resource;

pub struct ResourceContainer<R>
where
    R: Resource,
{
    pub(crate) resource: R,
    pub(crate) last_modified_tick: u64,
}
