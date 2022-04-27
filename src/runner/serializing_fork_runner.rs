use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::{SerializingForkRunner, Runner};
use crate::WatchdogConfig;


impl Runner for SerializingForkRunner {
    fn run(&self, _: &mut Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl SerializingForkRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
