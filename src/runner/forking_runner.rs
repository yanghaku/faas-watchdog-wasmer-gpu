use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::Runner;
use crate::WatchdogConfig;


#[derive(Clone)]
pub(crate) struct ForkingRunner;


impl Runner for ForkingRunner {
    fn run(&self, _: Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl ForkingRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
