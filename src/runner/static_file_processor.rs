use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::{StaticFileProcessor, Runner};
use crate::WatchdogConfig;


impl Runner for StaticFileProcessor {
    fn run(&self, _: &mut Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl StaticFileProcessor {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
