use crate::runner::Runner;
use crate::WatchdogConfig;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct SerializingForkRunner;

impl Runner for SerializingForkRunner {}

impl SerializingForkRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
