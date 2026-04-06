use std::hash::Hash;

pub trait AnimationState: Copy + Eq + Hash + 'static {
    fn count() -> u16;

    fn as_index(&self) -> u16;

    fn name(&self) -> &'static str;
}
