use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use super::watchdog_mode::WATCHDOG_MODE_STR;
use super::{WatchdogConfig, WatchdogMode};

#[cfg(feature = "wasm")]
use crate::runner::wasm_runner::*;

const KET_PORT: &str = "port";
const DEFAULT_PORT: u16 = 8080;

const KEY_READ_TIMEOUT: &str = "read_timeout";
const DEFAULT_READ_TIMEOUT_SEC: u64 = 10;

const KEY_WRITE_TIMEOUT: &str = "write_timeout";
const DEFAULT_WRITE_TIMEOUT_SEC: u64 = 10;
const KEY_HEALTH_CHECK_INTERVAL: &str = "healthcheck_interval";

const KEY_EXEC_TIMEOUT: &str = "exec_timeout";
const DEFAULT_EXEC_TIMEOUT_SEC: u64 = 10;

const KEY_MODE: &str = "mode";
const DEFAULT_MODE: WatchdogMode = WatchdogMode::ModeWasm;

const KEY_FUNC_NAME_1: &str = "function_process";
const KEY_FUNC_NAME_2: &str = "fprocess";
const KEY_UPSTREAM_URL_1: &str = "http_upstream_url";
const KEY_UPSTREAM_URL_2: &str = "upstream_url";

const KEY_CONTENT_TYPE: &str = "content_type";
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

const KEY_STATIC_PATH: &str = "static_path";
const DEFAULT_STATIC_PATH: &str = "/home/app/public";

const KEY_SUPPRESS_LOCK: &str = "suppress_lock";
const DEFAULT_SUPPRESS_LOCK: bool = false;

const KEY_MAX_INFLIGHT: &str = "max_inflight";
const DEFAULT_MAX_INFLIGHT: i32 = 0;

const KEY_BUFFER_HTTP_1: &str = "buffer_http";
const KEY_BUFFER_HTTP_2: &str = "http_buffer_req_body";
const DEFAULT_BUFFER_HTTP: bool = false;

const KEY_PREFIX_LOGS: &str = "prefix_logs";
const DEFAULT_PREFIX_LOGS: bool = true;

const KEY_LOG_BUFFER_SIZE: &str = "log_buffer_size";
const DEFAULT_LOG_BUFFER_SIZE: i32 = 65536;

pub(crate) const KEY_MIN_SCALE: &str = "min_scale";
pub(crate) const KEY_MAX_SCALE: &str = "max_scale";

const INJECT_CGI_HEADERS: bool = true;
const METRICS_PORT: u16 = 8081;

impl WatchdogConfig {
    // generate the instance of WatchdogConfig from the given environment variable
    pub(crate) fn new(vars: &HashMap<String, String>) -> Result<Self> {
        let tcp_port = parse_var(vars, &KET_PORT).unwrap_or(DEFAULT_PORT);

        let http_read_timeout = Duration::from_secs(
            parse_var(vars, KEY_READ_TIMEOUT).unwrap_or(DEFAULT_READ_TIMEOUT_SEC),
        );
        let http_write_timeout = Duration::from_secs(
            parse_var(vars, KEY_WRITE_TIMEOUT).unwrap_or(DEFAULT_WRITE_TIMEOUT_SEC),
        );
        if http_write_timeout.is_zero() {
            return Err(anyhow!("HTTP write timeout must be over 0s."));
        }

        let health_check_interval = match parse_var(vars, KEY_HEALTH_CHECK_INTERVAL) {
            Some(t) => Duration::from_secs(t),
            None => http_write_timeout,
        };

        let exec_timeout = Duration::from_secs(
            parse_var(vars, KEY_EXEC_TIMEOUT).unwrap_or(DEFAULT_EXEC_TIMEOUT_SEC),
        );

        let operational_mode = match vars.get(KEY_MODE) {
            Some(str) => {
                let mode = WatchdogMode::from(str);
                if mode == WatchdogMode::ModeUnknown {
                    let mut available_mode = String::new();
                    for i in 1..WATCHDOG_MODE_STR.len() {
                        available_mode += WATCHDOG_MODE_STR[i];
                        available_mode += ",";
                    }
                    return Err(anyhow!(
                        "unknown watchdog mode: {} \navailable mode is [{}]",
                        str,
                        available_mode
                    ));
                }
                mode
            }
            _ => env_get_or_warn!(None, KEY_MODE, DEFAULT_MODE),
        };

        let function_process = match vars.get(KEY_FUNC_NAME_1) {
            Some(str) => str.clone(),
            None => {
                match vars.get(KEY_FUNC_NAME_2) {
                    Some(str) => str.clone(),
                    None => {
                        if operational_mode == WatchdogMode::ModeStatic {
                            // the static mode does not need function name
                            String::default()
                        } else {
                            return Err(anyhow!(
                                "Please provide a \"function_process\" or \"fprocess\" \
                                    environmental variable for your function."
                            ));
                        }
                    }
                }
            }
        };

        let content_type =
            parse_var(vars, KEY_CONTENT_TYPE).unwrap_or(DEFAULT_CONTENT_TYPE.to_string());

        let upstream_url = match parse_var(vars, KEY_UPSTREAM_URL_1) {
            Some(u) => Some(u),
            None => parse_var(vars, KEY_UPSTREAM_URL_2),
        };

        let static_path =
            parse_var(vars, KEY_STATIC_PATH).unwrap_or(DEFAULT_STATIC_PATH.to_string());

        let suppress_lock = parse_var(vars, KEY_SUPPRESS_LOCK).unwrap_or(DEFAULT_SUPPRESS_LOCK);
        let max_inflight = parse_var(vars, KEY_MAX_INFLIGHT).unwrap_or(DEFAULT_MAX_INFLIGHT);

        let buffer_http_body = parse_var(vars, KEY_BUFFER_HTTP_1)
            .unwrap_or(parse_var(vars, KEY_BUFFER_HTTP_2).unwrap_or(DEFAULT_BUFFER_HTTP));

        let prefix_logs = parse_var(vars, KEY_PREFIX_LOGS).unwrap_or(DEFAULT_PREFIX_LOGS);
        let log_buffer_size =
            parse_var(vars, KEY_LOG_BUFFER_SIZE).unwrap_or(DEFAULT_LOG_BUFFER_SIZE);

        // check
        if operational_mode == WatchdogMode::ModeHTTP && upstream_url.is_none() {
            return Err(anyhow!(
                "For \"mode=http\" you must specify a valid URL for \"http_upstream_url\""
            ));
        }
        if operational_mode == WatchdogMode::ModeStatic && static_path == "" {
            return Err(anyhow!(
                "For mode=static you must specify the \"static_path\" to serve"
            ));
        }

        Ok(Self {
            _tcp_port: tcp_port,
            _http_read_timeout: http_read_timeout,
            _http_write_timeout: http_write_timeout,
            _exec_timeout: exec_timeout,
            _health_check_interval: health_check_interval,
            _function_process: function_process,
            _content_type: content_type,
            _inject_cgi_headers: INJECT_CGI_HEADERS,
            _operational_mode: operational_mode,
            _suppress_lock: suppress_lock,
            _upstream_url: upstream_url,
            _static_path: static_path,
            _buffer_http_body: buffer_http_body,
            _metrics_port: METRICS_PORT,
            _max_inflight: max_inflight,
            _prefix_logs: prefix_logs,
            _log_buffer_size: log_buffer_size,
            _min_scale: parse_var(vars, KEY_MIN_SCALE),
            _max_scale: parse_var(vars, KEY_MAX_SCALE),

            #[cfg(feature = "wasm")]
            _wasm_root: parse_var(vars, KEY_WASM_ROOT),
            #[cfg(feature = "wasm")]
            _wasm_c_target_triple: parse_var(vars, KEY_WASM_C_TARGET_TRIPLE),
            #[cfg(feature = "wasm")]
            _wasm_c_cpu_features: parse_var(vars, KEY_WASM_C_CPU_FEATURES),
            #[cfg(feature = "wasm")]
            _use_cuda: parse_var(vars, KEY_USE_CUDA),
        })
    }
}

#[inline]
fn parse_var<T>(vars: &HashMap<String, String>, key: &'static str) -> Option<T>
where
    T: FromStr,
{
    match vars.get(key) {
        Some(str) => match str.parse::<T>() {
            Ok(res) => Some(res),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::WatchdogConfig;
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default() {
        let keys = vec![KEY_FUNC_NAME_1, KEY_FUNC_NAME_2];

        for key in keys.iter() {
            let mut env = HashMap::new();
            let f_process = "process".to_string();
            env.insert(key.to_string(), f_process.clone());

            let cfg = WatchdogConfig::new(&env).expect("create default watchdog config error");

            assert_eq!(cfg._tcp_port, DEFAULT_PORT);
            assert_eq!(cfg._http_read_timeout.as_secs(), DEFAULT_READ_TIMEOUT_SEC);
            assert_eq!(cfg._http_write_timeout.as_secs(), DEFAULT_WRITE_TIMEOUT_SEC);
            assert_eq!(cfg._exec_timeout.as_secs(), DEFAULT_EXEC_TIMEOUT_SEC);
            assert_eq!(
                cfg._health_check_interval.as_secs(),
                DEFAULT_WRITE_TIMEOUT_SEC
            );
            assert_eq!(cfg._function_process, f_process);
            assert_eq!(cfg._content_type, DEFAULT_CONTENT_TYPE);
            assert_eq!(cfg._inject_cgi_headers, INJECT_CGI_HEADERS);
            assert_eq!(cfg._operational_mode, DEFAULT_MODE);
            assert_eq!(cfg._suppress_lock, DEFAULT_SUPPRESS_LOCK);
            assert_eq!(cfg._upstream_url, None);
            assert_eq!(cfg._static_path, DEFAULT_STATIC_PATH);
            assert_eq!(cfg._buffer_http_body, DEFAULT_BUFFER_HTTP);
            assert_eq!(cfg._metrics_port, METRICS_PORT);
            assert_eq!(cfg._max_inflight, DEFAULT_MAX_INFLIGHT);
            assert_eq!(cfg._prefix_logs, DEFAULT_PREFIX_LOGS);
            assert_eq!(cfg._log_buffer_size, DEFAULT_LOG_BUFFER_SIZE);
            assert_eq!(cfg._min_scale, None);
            assert_eq!(cfg._max_scale, None);
            #[cfg(feature = "wasm")]
            assert_eq!(cfg._wasm_root, None);
            #[cfg(feature = "wasm")]
            assert_eq!(cfg._use_cuda, None);
            #[cfg(feature = "wasm")]
            assert_eq!(cfg._wasm_c_target_triple, None);
            #[cfg(feature = "wasm")]
            assert_eq!(cfg._wasm_c_cpu_features, None);
        }
    }

    #[test]
    fn test_empty_error() {
        let env = HashMap::new();
        let cfg = WatchdogConfig::new(&env);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_static_mode() {
        let mut env = HashMap::new();
        env.insert(KEY_MODE.to_string(), "static".to_string());
        let cfg = WatchdogConfig::new(&env).expect("create static mode watchdog config error");
        assert_eq!(cfg._function_process, String::default());
        assert_eq!(cfg._operational_mode, WatchdogMode::ModeStatic);
    }

    #[test]
    fn test_write_timeout_error() {
        let mut env = HashMap::new();
        env.insert(KEY_WRITE_TIMEOUT.to_string(), "0".to_string());
        let cfg = WatchdogConfig::new(&env);
        assert!(cfg.is_err());
    }
}
