use anyhow::Result;
use crate::runner::wasm_runner::Compiler;


impl Compiler {
    pub(crate) fn default() -> Self {
        Self {}
    }

    pub(crate) fn do_compile(&self, in_file: &String, out_file: &String) -> Result<()> {
        eprintln!("compile {} to {}", in_file, out_file);
        Ok(())
    }
}
