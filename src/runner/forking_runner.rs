use crate::runner::Runner;
use crate::WatchdogConfig;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct ForkingRunner;

impl Runner for ForkingRunner {}

impl ForkingRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
