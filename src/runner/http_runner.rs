use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::{HttpRunner, Runner};
use crate::WatchdogConfig;


impl Runner for HttpRunner {
    fn run(&self, _: &mut Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}


impl HttpRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
