use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use log::{info, warn};
use wasmer::{CpuFeature, Dylib, DylibArtifact, Engine, LLVM, Module, Store, Target, Triple};

use crate::runner::wasm_runner::Compiler;


/// The implementation for webassembly compiler wrapper
/// default engine is Dylib
/// default compiler is LLVM
impl Compiler {
    /// new compiler for given target triple and cpu_features
    pub(crate) fn new(target_triple: Option<String>, cpu_features: Option<String>) -> Result<Self> {
        // parse the target or use default native target
        let target = Self::parse_target(target_triple, cpu_features)?;

        // new llvm compiler config
        let compiler_config = LLVM::new();

        // new dylib engine
        let engine = Dylib::new(compiler_config).target(target).engine();

        Ok(Self {
            _store: Store::new(&engine),
            _out_extension: DylibArtifact::get_default_extension(engine.target().triple()),
        })
    }


    /// if the wasm module has cached native binary file, return the deserialize module
    /// else do compile and return the compiled module
    /// todo: add safety strategy for cached file
    pub(crate) fn try_load_cached(&self, wasm_file: &PathBuf) -> Result<Module> {
        let mut cached_file = wasm_file.clone();
        cached_file.set_extension(self._out_extension);

        // judge if cached file exists and valid
        if cached_file.is_file() {
            // try deserialize the module from file
            match unsafe { Module::deserialize_from_file(&self._store, &cached_file) } {
                Ok(module) => {
                    info!("deserialize module from cached binary file success");
                    return Ok(module);
                }
                Err(e) => {
                    warn!("cached wasm binary file exist, but can not be loaded! error = {:?}", e);
                }
            }
        }

        info!("compiling the webassembly module");
        let wasm_bytes = fs::read(wasm_file)?;
        let (module, duration) = self.do_compile(&wasm_bytes)?;
        info!("compile success, usage {} ms", duration.as_millis());

        // try to serialize the module and save to cached file
        match module.serialize_to_file(&cached_file) {
            Ok(_) => {
                info!("serialize the module and save to cached file success");
            }
            Err(e) => {
                warn!("serialize the module and save to cached file fail! error = {:?}", e);
            }
        }

        Ok(module)
    }


    /// do the compile stage, compile the wasm bytes to native code and return time duration
    #[inline(always)]
    pub(crate) fn do_compile(&self, bytes: &[u8]) -> Result<(Module, Duration)> {
        let start_time = SystemTime::now();

        let module = Module::from_binary(&self._store, bytes)?;

        let end_time = SystemTime::now();
        Ok((module, end_time.duration_since(start_time).unwrap()))
    }


    /// compile from wasm file to dylib file
    #[inline(always)]
    pub(crate) fn compile_to_file(&self, in_file: &String, out_file: &String) -> Result<()> {
        // load wasm module file
        let wasm_bytes = fs::read(in_file)?;

        // do compile
        let (module, duration) = self.do_compile(&wasm_bytes)?;

        // serialize to file
        let binary = module.serialize()?;
        let out_path = PathBuf::from(out_file);
        fs::write(out_file, binary)?;


        // check the out file extension
        let out_filename = out_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        match out_path.extension() {
            Some(ext) => {
                if ext != self._out_extension {
                    warn!("The output file has a wrong extension. \
                    We recommend using `{}.{}` for the chosen target", out_filename, &self._out_extension);
                }
            }
            None => {
                warn!("The output file has no extension. \
                We recommend using `{}.{}` for the chosen target", out_filename, &self._out_extension);
            }
        }

        info!("Compile {} to {} success! \nTime usage = {} ms", in_file, out_file, duration.as_millis());
        Ok(())
    }


    fn parse_target(triple_opt: Option<String>, cpu_features_str: Option<String>) -> Result<Target> {
        let triple = match triple_opt {
            None => Triple::host(),
            Some(triple_str) => {
                triple_str.parse::<Triple>()
                    .map_err(|e| {
                        anyhow!("Parse the target triple error: {}", e.to_string())
                    })?
            }
        };

        let cpu_features = match cpu_features_str {
            None => CpuFeature::for_host(),
            Some(_) => {
                todo!()
            }
        };

        Ok(Target::new(triple, cpu_features))
    }
}


#[cfg(test)]
mod test {
    use wasmer::Target;
    use crate::Compiler;

    #[test]
    fn test_default() {
        let store = Compiler::new(None, None).unwrap()._store;
        let engine = store.engine();
        assert_eq!(engine.target().clone(), Target::default());
    }

    #[test]
    fn test_triples() {
        let triples = vec!["aarch64-apple-darwin", "x86_64-unknown-linux-gnu", "i386-pc-windows-msvc"];
        let extensions = vec!["dylib", "so", "dll"];

        for i in 0..triples.len() {
            let compiler = Compiler::new(
                Some(triples[i].to_string()), None);
            assert!(compiler.is_ok());
            assert_eq!(compiler.unwrap()._out_extension, extensions[i]);
        }
    }
}
