use std::path::PathBuf;
use anyhow::Result;
use hyper::{Body, Request, Response};
use log::{debug, warn};
use wasmer::Store;
use wasmer_wasi::WasiState;

use super::{Runner, WasmRunner};
use super::utils::parse_command;
use crate::WatchdogConfig;


/// compile the wasm module to native dylib
mod compiler;


pub(crate) const DEFAULT_WASM_ROOT: &str = "/wasm_root";
pub(crate) const KEY_WASM_ROOT: &str = "wasm_root";
pub(crate) const KEY_WASM_C_TARGET_TRIPLE: &str = "wasm_c_target";
pub(crate) const KEY_WASM_C_CPU_FEATURES: &str = "wasm_c_cpu_features";
const BIN_DIR: &str = "bin";
const RUN_DIR: &str = "run";


pub(crate) struct Compiler {
    _store: Store,
    _out_extension: &'static str,
}


impl Runner for WasmRunner {
    fn run(&self, req: &mut Request<Body>, res: &mut Response<Body>) -> Result<()> {
        // build the wasi environment
        let mut wasi_env = WasiState::new(self._func_process[0].as_str())
            .args(&self._func_process[1..self._func_process.len()])
            .map_dir("/", self._wasm_root.join(RUN_DIR))?
            .finalize()?;

        let import_object = wasi_env.import_object(&self._module)?;

        // instate the wasm
        let instance = wasmer::Instance::new(&self._module, &import_object)?;

        // get start function
        let m = instance.exports.get_function("_start")?;


        // call the start function
        m.call(&[])?;

        Ok(())
    }
}


impl WasmRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        let wasm_root = PathBuf::from(match config._wasm_root {
            None => {
                warn!("{} is not specified, use the default path: {}", KEY_WASM_ROOT, DEFAULT_WASM_ROOT);
                DEFAULT_WASM_ROOT.to_string()
            }
            Some(r) => r
        });

        let func_process = parse_command(&config._function_process)?;

        let mut module_path = wasm_root.join(BIN_DIR).join(func_process[0].as_str());
        // only use wasm as the function extension
        module_path.set_extension("wasm");
        debug!("webassembly module path is {}", module_path.display());

        let compiler = Compiler::new(config._wasm_c_target_triple, config._wasm_c_cpu_features)?;
        let module = compiler.try_load_cached(&module_path)?;

        Ok(Self {
            _module: module,
            _func_process: func_process,
            _wasm_root: wasm_root,
        })
    }
}
