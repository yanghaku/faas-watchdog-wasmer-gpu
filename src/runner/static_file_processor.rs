use crate::runner::Runner;
use crate::WatchdogConfig;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct StaticFileProcessor;

impl Runner for StaticFileProcessor {}

impl StaticFileProcessor {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
