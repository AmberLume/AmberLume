use crate::render::factories::buffer::managed_buffer::ManagedBuffer;

pub trait IntoBuffer<T> {
    type Output;

    fn into_buffer(self, handle: ManagedBuffer) -> Self::Output;
}
