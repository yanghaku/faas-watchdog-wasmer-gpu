use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::{ForkingRunner, Runner};
use crate::WatchdogConfig;


impl Runner for ForkingRunner {
    fn run(&self, _: &mut Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl ForkingRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
