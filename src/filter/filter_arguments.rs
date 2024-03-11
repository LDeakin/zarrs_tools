use super::{FilterCommonArguments, FilterError, FilterTraits};

pub trait FilterArguments {
    fn name(&self) -> String;

    fn init(
        &self,
        common_args: &FilterCommonArguments,
    ) -> Result<Box<dyn FilterTraits>, FilterError>;
}
