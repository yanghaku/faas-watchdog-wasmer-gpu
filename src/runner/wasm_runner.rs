/// compile the wasm module to native dylib
mod compiler;

/// for running the functions
mod thread_pool;

/// the virtual file system for stdin/stdout/stderr
mod stdio;


use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use hyper::{Body, Request, Response};
use log::{debug, info, warn};
use wasmer_wasi::WasiState;

use super::Runner;
use super::utils::parse_command;
use crate::WatchdogConfig;
pub(crate) use compiler::Compiler;
use stdio::{Stdin, Stdout, Stderr};
use thread_pool::ThreadPool;


pub(crate) const DEFAULT_WASM_ROOT: &str = "/wasm_root";
pub(crate) const KEY_WASM_ROOT: &str = "wasm_root";
pub(crate) const KEY_WASM_C_TARGET_TRIPLE: &str = "wasm_c_target";
pub(crate) const KEY_WASM_C_CPU_FEATURES: &str = "wasm_c_cpu_features";
const DEFAULT_MAX_SCALE: usize = 1024;
const BIN_DIR: &str = "bin";
const RUN_DIR: &str = "run";


/// The data for wasm runner
struct WasmRunnerEntry {
    /// the thread pool to run functions
    _worker: ThreadPool,

    /// the function process and arguments
    _func_process: Vec<String>,

    /// the max scale number
    _max_scale: usize,

    /// if log prefix has prefix
    _log_prefix: bool,

    /// log buffer size
    _log_buffer_size: usize,

    /// compiled wasm module
    _module: wasmer::Module,

    /// workplace root directory
    _wasm_root: PathBuf,
}


/// [```WasmRunner```]
/// run the function request in WebAssembly
#[cfg(feature = "wasm")]
#[derive(Clone)]
pub(crate) struct WasmRunner {
    _inner: Arc<WasmRunnerEntry>,
}


impl Runner for WasmRunner {
    fn run(&self, req: Request<Body>, res: &mut Response<Body>) -> Result<()> {
        let (sender, receiver) = channel();

        let runner = self.clone();
        // run function in thread pool
        self._inner._worker.execute(move || {
            // send the run result
            sender.send(runner.run_inner(req)).unwrap();
        });

        // wait for result from thread pool
        let res_body = receiver.recv()?;

        // try get response body
        *res.body_mut() = res_body?;

        Ok(())
    }

    fn scale(&self, replicas: usize) -> Result<()> {
        if replicas == 0 {
            Err(anyhow!("Replicas can not be zero!"))
        } else if replicas > self._inner._max_scale {
            Err(anyhow!("Replicas can not greater than `{}`!", self._inner._max_scale))
        } else {
            self._inner._worker.set_thread_num(replicas);
            info!("Wasm runner set the replicas to `{}`", replicas);
            Ok(())
        }
    }
}


impl WasmRunner {
    /// create a new wasm runner
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        let wasm_root = PathBuf::from(match config._wasm_root {
            None => {
                warn!("The environment variable `{}` is not specified, use the default path: `{}`", KEY_WASM_ROOT, DEFAULT_WASM_ROOT);
                DEFAULT_WASM_ROOT.to_string()
            }
            Some(r) => r
        });
        let max_scale = match config._max_scale {
            Some(m) => m,
            None => {
                warn!("The environment variable `max_scale` is not specified, use the default value: `{}`", DEFAULT_MAX_SCALE);
                DEFAULT_MAX_SCALE
            }
        };
        let log_buffer_size = if config._log_buffer_size <= 0 {
            0 as usize
        } else {
            config._log_buffer_size as usize
        };


        let mut func_process = parse_command(&config._function_process)?;

        let mut module_path = wasm_root.join(BIN_DIR).join(func_process[0].as_str());
        // only use wasm as the function extension
        module_path.set_extension("wasm");
        func_process[0] = module_path.display().to_string();
        debug!("Webassembly module path is `{}`", func_process[0]);

        let compiler = Compiler::new(config._wasm_c_target_triple, config._wasm_c_cpu_features)?;
        let module = compiler.try_load_cached(&module_path)?;

        // default use cpu's numbers as thread pool's thread number
        let thread_num = num_cpus::get();

        let thread_pool = ThreadPool::new(thread_num, Some(func_process[0].clone()), None);

        Ok(Self {
            _inner: Arc::new(WasmRunnerEntry {
                _worker: thread_pool,
                _log_prefix: config._prefix_logs,
                _log_buffer_size: log_buffer_size,
                _max_scale: max_scale,
                _func_process: func_process,
                _module: module,
                _wasm_root: wasm_root,
            })
        })
    }


    /// run the function in thread pool
    /// return the stdout as response body
    pub(crate) fn run_inner(&self, req: Request<Body>) -> Result<Body> {
        let start_time = SystemTime::now();
        let thread_id = thread::current().id();
        let func_process = &self._inner._func_process;

        // init the stdio for function
        let stdin = Box::new(Stdin::new(req.into_body())?);
        let stdout = Box::new(Stdout::new());

        let stderr = Box::new(Stderr::new(
            format!("{:?}-`{}`", thread_id, func_process[0]),
            self._inner._log_prefix,
            self._inner._log_buffer_size)
        );

        // build the wasi environment
        let mut wasi_env = WasiState::new(func_process[0].as_str())
            .args(&func_process[1..func_process.len()])
            .map_dir("/", self._inner._wasm_root.join(RUN_DIR))?
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .env("PWD", "/")
            .finalize()?;

        let import_object = wasi_env.import_object(&self._inner._module)?;

        // instate the wasm
        let instance = wasmer::Instance::new(&self._inner._module, &import_object)?;

        // get start function
        let m = instance.exports.get_function("_start")?;

        // call the start function
        m.call(&[])?;

        let duration = SystemTime::now().duration_since(start_time).unwrap();
        info!("{:?} run function `{}` took {} us  ({} ms)", thread_id, func_process[0],
            duration.as_micros(), duration.as_millis());

        // read stdout to response body
        if let Some(wasi_stdout_box) = wasi_env.state().fs.stdout_mut()? {
            if let Some(wasi_stdout) = wasi_stdout_box.downcast_mut::<Stdout>() {
                return Ok(Body::from(wasi_stdout.take_buffer()));
            }
        }
        Err(anyhow!("Cannot find the wasi `stdout` handler"))
    }
}
