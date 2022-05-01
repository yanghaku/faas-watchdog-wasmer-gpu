// Copyright [2022] [bo.yang@smail.nju.edu.cn]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


/// read the watch config from environment
mod config;

/// health check
mod health;

/// runner (such as http mode, wasm mode)
mod runner;

/// metrics
mod metrics;

/// http server for watchdog
mod server;

/// some help function
mod utils;


extern crate lazy_static;

use std::collections::HashMap;
use std::io::Write;
use std::process::exit;
use std::env::args;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use chrono::{DateTime, SecondsFormat};
use log::{debug, error, info};

use server::start_server;
pub(crate) use config::{WatchdogConfig, WatchdogMode};
pub(crate) use health::*;
pub(crate) use utils::*;

#[cfg(feature = "compiler")]
use crate::runner::wasm_runner::{Compiler, KEY_WASM_C_CPU_FEATURES, KEY_WASM_C_TARGET_TRIPLE};


/// main function for watchdog
fn main() {
    // set up log
    let log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let logger_env = env_logger::Env::default().default_filter_or(log_level);
    env_logger::Builder::from_env(logger_env).format(|buf, record| {
        let now = DateTime::from(SystemTime::now()).to_rfc3339_opts(SecondsFormat::Millis, true);
        writeln!(buf, "[watchdog {} {}] {}", now, record.level(), record.args())
    }).init();

    let exit_code = match run(&args().collect(), environment_vars()) {
        Ok(_) => 0,
        Err(e) => {
            error!("{}", e);
            1
        }
    };

    info!("Watchdog exit with status {}", exit_code);

    exit(exit_code);
}


/// process the argument with given environment variables
fn run(args: &Vec<String>, env: &HashMap<String, String>) -> Result<()> {
    let bin_path = args.get(0).ok_or(
        anyhow!("Cannot resolve the first argument"))?;

    match args.get(1).unwrap_or(&"".to_string()).as_str() {
        #[cfg(feature = "compiler")]
        "-c" | "--compile" => {
            let in_file = args.get(2);
            let out_opt = args.get(3);
            let out_file = args.get(4);

            if in_file.is_none() || out_file.is_none() ||
                out_opt.is_none() || out_opt.unwrap().as_str().ne("-o") {
                // print help msg and report syntax error
                print_helper(bin_path);
                return if in_file.is_none() {
                    Err(anyhow!("The following required arguments were not provided:\n\
                      <IN_FILE> -o <OUT_FILE>\n"))
                } else {
                    Err(anyhow!("The following required arguments were not provided:\n\
                      -o <OUT_FILE>\n"))
                };
            }
            let triple = env.get(KEY_WASM_C_TARGET_TRIPLE).cloned();
            let cpu_features = env.get(KEY_WASM_C_CPU_FEATURES).cloned();
            return Compiler::new(triple, cpu_features)?
                .compile_to_file(in_file.unwrap(), out_file.unwrap());
        }

        "-v" | "--version" => {
            print_version();
        }

        "-h" | "--help" => {
            print_helper(bin_path);
        }

        "--run-healthcheck" => {
            return if lock_file_present() {
                Ok(())
            } else {
                Err(anyhow!("Unable to find lock file."))
            };
        }

        _ => { // start the watchdog server and metrics server
            print_version();

            let watchdog_config = WatchdogConfig::new(env)?;
            debug!("{:?}", watchdog_config);

            mark_healthy(watchdog_config._suppress_lock)?;
            let res = start_server(watchdog_config);
            mark_unhealthy()?;

            if res.is_err() {
                return res;
            }
        }
    };

    Ok(())
}


/// Get version and git commit sha-1 in build time
#[inline(always)]
fn get_version() -> (&'static str, &'static str) {
    const GIT_COMMIT_SHA: Option<&str> = option_env!("GIT_COMMIT_SHA");
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    const UNKNOWN: &str = "unknown";
    (VERSION.unwrap_or(UNKNOWN), GIT_COMMIT_SHA.unwrap_or(UNKNOWN))
}


/// print the version
#[inline(always)]
fn print_version() {
    let (version, git_sha) = get_version();
    println!("Version: {}\tSHA: {}\n", version, git_sha);
}


/// print the help message
#[inline(always)]
fn print_helper(bin_path: &String) {
    #[cfg(feature = "compiler")]
    println!("usage: {} [-c, --compile <IN_FILE> -o <OUT_FILE> ] [-v, --version] [-h, --help] [--run-healthcheck]", bin_path);

    #[cfg(not(feature = "compiler"))]
    println!("usage: {} [-v, --version] [-h, --help] [--run-healthcheck]", bin_path);

    println!("optional arguments:");

    #[cfg(feature = "compiler")]
    println!("  -c, --compile <IN_FILE> -o <OUT_FILE>    Compile the wasm module to dylib and exit.");

    println!("  -v, --version                            Print the version and exit.");
    println!("  -h, --help                               Print the help information and exit.");
    // for watchdog
    println!("      --run-healthcheck                    Check for the a lock-file, when using an exec health check. \
                                                         Exit 0 for present, non-zero when not found.");
}
