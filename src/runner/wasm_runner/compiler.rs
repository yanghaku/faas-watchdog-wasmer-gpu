#[cfg(feature = "compiler")]
use std::fs;

use std::path::PathBuf;

#[cfg(feature = "compiler")]
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use log::{info, warn};
use wasmer::{Dylib, DylibArtifact, Module, Store, Triple};

#[cfg(feature = "compiler")]
use wasmer::{CpuFeature, Engine, LLVM, Target};

pub(crate) struct Compiler {
    _store: Store,
    _out_extension: &'static str,
}


/// The implementation for webassembly compiler wrapper
/// default engine is Dylib
/// default compiler is LLVM
impl Compiler {
    #[cfg(feature = "compiler")]
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


    #[cfg(not(feature = "compiler"))]
    /// Create new compiler with headless engine
    pub(crate) fn new(target_triple: Option<String>, cpu_features: Option<String>) -> Result<Self> {
        if target_triple.is_some() {
            warn!("No Compiler! environment variable `{}` is set but not used", super::KEY_WASM_C_TARGET_TRIPLE);
        }
        if cpu_features.is_some() {
            warn!("No Compiler! environment variable `{}` is set but not used", super::KEY_WASM_C_CPU_FEATURES);
        }


        let engine = Dylib::headless().engine();
        Ok(Self {
            _store: Store::new(&engine),
            _out_extension: DylibArtifact::get_default_extension(&Triple::host()),
        })
    }


    /// if the wasm module has been compiled to native binary file, return the deserialize module
    /// else do compile and return the compiled module
    /// todo: add safety strategy for cached file
    #[allow(unused_mut)]
    pub(crate) fn try_load_compiled(&self, mut wasm_file: PathBuf) -> Result<Module> {
        #[cfg(feature = "compiler")]
            let mut compiled_file = wasm_file.clone();
        #[cfg(not(feature = "compiler"))]
            let mut compiled_file = wasm_file; // just move

        compiled_file.set_extension(self._out_extension);

        // judge if cached file exists and valid
        if compiled_file.is_file() {
            // try deserialize the module from file
            match unsafe { Module::deserialize_from_file(&self._store, &compiled_file) } {
                Ok(module) => {
                    info!("Deserialize module from cached binary file success");
                    return Ok(module);
                }
                Err(e) => {
                    warn!("Compiled wasm module file `{}` exist, but can not be loaded! error = {:?}",
                        compiled_file.display(), e);
                }
            }
        }


        #[cfg(feature = "compiler")]
        return {
            info!("Compiling the webassembly module");

            wasm_file.set_extension("wasm");
            let wasm_bytes = fs::read(wasm_file)?;
            let (module, duration) = self.do_compile(&wasm_bytes)?;
            info!("Compile success, usage {} ms", duration.as_millis());

            // try to serialize the module and save to cached file
            match module.serialize_to_file(&compiled_file) {
                Ok(_) => {
                    info!("Serialize the module and save to module file success");
                }
                Err(e) => {
                    warn!("Serialize the module and save to module file fail! error = {:?}", e);
                }
            }

            Ok(module)
        };

        // if no compiler, just return error msg
        #[cfg(not(feature = "compiler"))]
        return {
            if !compiled_file.is_file() {
                log::error!("Cannot find the webassembly file `{}`", compiled_file.display());
            }
            Err(anyhow!("Deserialize module fail and no compiler feature enable"))
        };
    }


    /// do the compile stage, compile the wasm bytes to native code and return time duration
    #[inline(always)]
    #[cfg(feature = "compiler")]
    pub(crate) fn do_compile(&self, bytes: &[u8]) -> Result<(Module, Duration)> {
        let start_time = SystemTime::now();

        let module = Module::from_binary(&self._store, bytes)?;

        let end_time = SystemTime::now();
        Ok((module, end_time.duration_since(start_time).unwrap()))
    }


    /// compile from wasm file to dylib file
    #[inline(always)]
    #[cfg(feature = "compiler")]
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


    #[cfg(feature = "compiler")]
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
    use super::Compiler;

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
