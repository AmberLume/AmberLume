use std::hash::Hash;

pub trait AnimationState: Copy + Eq + Hash + 'static {
    fn count() -> u16;

    fn as_index(&self) -> u16;

    fn from_index(index: u16) -> Self;

    fn name(&self) -> &'static str;
}
