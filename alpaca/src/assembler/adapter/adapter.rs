use anyhow::Result;

pub trait ResourceAdapter {
    type Input<'a>;

    type Output;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output>;
}
