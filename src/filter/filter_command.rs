use clap::{Parser, Subcommand};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::{
    filter_error::FilterError, filter_traits::FilterTraits, filters, FilterArguments,
    FilterCommonArguments, FilterInputOutputArguments,
};

#[derive(Debug, Clone, Parser, Deserialize)]
pub struct FilterCombinedArgs<TArgs: FilterArguments + Serialize + clap::Args> {
    #[command(flatten)]
    #[serde(flatten)]
    input_output: FilterInputOutputArguments,
    #[command(flatten)]
    #[serde(flatten)]
    args: TArgs,
    #[command(flatten)]
    #[serde(flatten)]
    common_args: FilterCommonArguments,
}

#[enum_dispatch]
pub trait FilterCommandTraits {
    fn name(&self) -> String;
    fn args_str(&self) -> String;
    fn reencode_str(&self) -> String {
        serde_json::to_string(&self.common_args().reencode()).unwrap()
    }
    fn io_args(&self) -> &FilterInputOutputArguments;
    fn common_args(&self) -> &FilterCommonArguments;
    fn common_args_mut(&mut self) -> &mut FilterCommonArguments;
    fn init(&self) -> Result<Box<dyn FilterTraits>, FilterError>;
}

impl<TArgs: FilterArguments + Serialize + clap::Args> FilterCommandTraits
    for FilterCombinedArgs<TArgs>
{
    fn name(&self) -> String {
        self.args.name()
    }

    fn args_str(&self) -> String {
        serde_json::to_string(&self.args).unwrap()
    }

    fn io_args(&self) -> &FilterInputOutputArguments {
        &self.input_output
    }

    fn common_args(&self) -> &FilterCommonArguments {
        &self.common_args
    }

    fn common_args_mut(&mut self) -> &mut FilterCommonArguments {
        &mut self.common_args
    }

    fn init(&self) -> Result<Box<dyn FilterTraits>, FilterError> {
        self.args.init(&self.common_args)
    }
}

#[derive(Debug, Clone, Subcommand, Deserialize)]
#[serde(tag = "filter", rename_all = "snake_case")]
#[enum_dispatch(FilterCommandTraits)]
pub enum FilterCommand {
    /// Reencode an array.
    Reencode(FilterCombinedArgs<filters::reencode::ReencodeArguments>),
    /// Crop an array given an offset and shape.
    Crop(FilterCombinedArgs<filters::crop::CropArguments>),
    /// Rescale array values given a multiplier and offset.
    Rescale(FilterCombinedArgs<filters::rescale::RescaleArguments>),
    /// Clamp values between a minimum and maximum.
    Clamp(FilterCombinedArgs<filters::clamp::ClampArguments>),
    /// Return a binary image where the input is equal to some value.
    Equal(FilterCombinedArgs<filters::equal::EqualArguments>),
    /// Downsample an image given a stride.
    Downsample(FilterCombinedArgs<filters::downsample::DownsampleArguments>),
    /// Compute the gradient magnitude.
    GradientMagnitude(FilterCombinedArgs<filters::gradient_magnitude::GradientMagnitudeArguments>),
    /// Apply a Gaussian kernel.
    Gaussian(FilterCombinedArgs<filters::gaussian::GaussianArguments>),
    /// Compute a summed area table (integral image).
    SummedAreaTable(FilterCombinedArgs<filters::summed_area_table::SummedAreaTableArguments>),
    /// Apply a guided filter (edge-preserving noise filter).
    GuidedFilter(FilterCombinedArgs<filters::guided_filter::GuidedFilterArguments>),
    /// Replace a value with another value.
    ReplaceValue(FilterCombinedArgs<filters::replace_value::ReplaceValueArguments>),
}
