use crate::runner::Runner;
use crate::WatchdogConfig;
use anyhow::Result;

#[derive(Clone)]
pub(crate) struct HttpRunner;

impl Runner for HttpRunner {}

impl HttpRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
