#[cfg(feature = "wasm")]
pub(crate) mod wasm_runner;


pub(crate) struct FunctionRequest {}


pub(crate) trait Runner {
    fn run(&mut self, _: &mut FunctionRequest);
}
