use anyhow::Result;

pub trait ResourceCollector {
    type Input<'a>;

    type Output;

    fn collect<'a>(&mut self, input: &Self::Input<'a>) -> Result<String>;

    fn get_resources(&self) -> Vec<(String, &Self::Output)>;
}
