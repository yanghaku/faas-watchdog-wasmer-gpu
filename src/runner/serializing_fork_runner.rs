use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::Runner;
use crate::WatchdogConfig;


#[derive(Clone)]
pub(crate) struct SerializingForkRunner;


impl Runner for SerializingForkRunner {
    fn run(&self, _: Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl SerializingForkRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
