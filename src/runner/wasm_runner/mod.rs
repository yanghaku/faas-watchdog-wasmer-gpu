use crate::runner::{FunctionRequest, Runner};

/// compile the wasm module to native dylib
mod compiler;

pub(crate) struct Compiler {}


pub(crate) struct WasmRunner {}


impl Runner for WasmRunner {
    fn run(&mut self, _: &mut FunctionRequest) {
        todo!()
    }
}

impl WasmRunner {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
