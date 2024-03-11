use clap::Parser;
use serde::Deserialize;

use crate::ZarrReencodingArgs;

#[derive(Debug, Clone, Parser, Deserialize)]
pub struct FilterCommonArguments {
    /// Reencoding args.
    #[command(flatten)]
    #[serde(flatten)]
    reencode: ZarrReencodingArgs,
    /// The maximum number of chunks concurrently processed.
    /// Inherits the top level arg if left unset.
    #[arg(long)]
    chunk_limit: Option<usize>,
}

impl FilterCommonArguments {
    pub fn reencode(&self) -> &ZarrReencodingArgs {
        &self.reencode
    }

    pub fn chunk_limit(&self) -> &Option<usize> {
        &self.chunk_limit
    }

    pub fn chunk_limit_mut(&mut self) -> &mut Option<usize> {
        &mut self.chunk_limit
    }
}
