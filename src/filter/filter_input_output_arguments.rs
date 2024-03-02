use clap::Parser;
use serde::Deserialize;

use super::path_or_identifier::{parse_path_or_identifier, PathOrIdentifier};

#[derive(Debug, Clone, Parser, Deserialize)]
pub struct FilterInputOutputArguments {
    /// Path to zarr input array.
    #[arg(required = true, value_parser = parse_path_or_identifier)]
    input: Option<PathOrIdentifier>,
    /// Path to zarr output array.
    #[arg(required = true, value_parser = parse_path_or_identifier)]
    output: Option<PathOrIdentifier>,
}

impl FilterInputOutputArguments {
    pub fn input(&self) -> &Option<PathOrIdentifier> {
        &self.input
    }

    pub fn output(&self) -> &Option<PathOrIdentifier> {
        &self.output
    }
}
