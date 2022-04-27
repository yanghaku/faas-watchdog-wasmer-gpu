use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use anyhow::{Result, Error};

use crate::config::{WatchdogConfig, WatchdogMode};
use crate::config::watchdog_mode::WATCHDOG_MODE_STR;


const DEFAULT_PORT: u16 = 8080;
const DEFAULT_READ_TIMEOUT_SEC: u64 = 10;
const DEFAULT_WRITE_TIMEOUT_SEC: u64 = 10;
const DEFAULT_EXEC_TIMEOUT_SEC: u64 = 10;
const DEFAULT_MODE: WatchdogMode = WatchdogMode::ModeStreaming;
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";
const DEFAULT_STATIC_PATH: &str = "/home/app/public";
const DEFAULT_WASM_ROOT: &str = "/wasm_root";
const DEFAULT_SUPPRESS_LOCK: bool = false;
const DEFAULT_MAX_INFLIGHT: i32 = 0;
const DEFAULT_BUFFER_HTTP: bool = false;
const DEFAULT_PREFIX_LOGS: bool = true;
const DEFAULT_LOG_BUFFER_SIZE: i32 = 65536;

const INJECT_CGI_HEADERS: bool = true;
const METRICS_PORT: u16 = 8081;


impl WatchdogConfig {
    // generate the instance of WatchdogConfig from the given environment variable
    pub(crate) fn new(vars: HashMap<String, String>) -> Result<Self> {
        let tcp_port = parse_var(&vars, "port").unwrap_or(DEFAULT_PORT);

        let http_read_timeout = Duration::from_secs(
            parse_var(&vars, "read_timeout").unwrap_or(DEFAULT_READ_TIMEOUT_SEC));
        let http_write_timeout = Duration::from_secs(
            parse_var(&vars, "write_timeout").unwrap_or(DEFAULT_WRITE_TIMEOUT_SEC));
        if http_write_timeout.is_zero() {
            return Err(Error::msg("HTTP write timeout must be over 0s."));
        }

        let health_check_interval = match parse_var(&vars, "healthcheck_interval") {
            Some(t) => Duration::from_secs(t),
            None => http_write_timeout
        };

        let exec_timeout = Duration::from_secs(
            parse_var(&vars, "exec_timeout").unwrap_or(DEFAULT_EXEC_TIMEOUT_SEC));

        let operational_mode = match vars.get("mode") {
            Some(str) => {
                let mode = WatchdogMode::from(str);
                if mode == WatchdogMode::ModeUnknown {
                    let mut available_mode = String::new();
                    for i in 1..WATCHDOG_MODE_STR.len() {
                        available_mode += WATCHDOG_MODE_STR[i];
                        available_mode += ",";
                    }
                    return Err(Error::msg(format!(
                        "unknown watchdog mode: {} \navailable mode is [{}]", str, available_mode)));
                }
                mode
            }
            _ => DEFAULT_MODE
        };

        let function_process = match vars.get("function_process") {
            Some(str) => str.clone(),
            None => {
                match vars.get("fprocess") {
                    Some(str) => str.clone(),
                    None => {
                        if operational_mode == WatchdogMode::ModeStatic {
                            // the static mode does not need function name
                            String::default()
                        } else {
                            return Err(Error::msg(
                                "Please provide a \"function_process\" or \"fprocess\" \
                                    environmental variable for your function."));
                        }
                    }
                }
            }
        };

        let content_type = parse_var(&vars, "content_type")
            .unwrap_or(DEFAULT_CONTENT_TYPE.to_string());

        let upstream_url = match parse_var(&vars, "http_upstream_url") {
            Some(u) => Some(u),
            None => parse_var(&vars, "upstream_url")
        };

        let static_path = parse_var(&vars, "static_path").unwrap_or(
            DEFAULT_STATIC_PATH.to_string());

        let wasm_root = parse_var(&vars, "wasm_root").unwrap_or(
            DEFAULT_WASM_ROOT.to_string());

        let suppress_lock = parse_var(&vars, "suppress_lock").unwrap_or(DEFAULT_SUPPRESS_LOCK);
        let max_inflight = parse_var(&vars, "max_inflight").unwrap_or(DEFAULT_MAX_INFLIGHT);

        let buffer_http_body = parse_var(&vars, "buffer_http").unwrap_or(
            parse_var(&vars, "http_buffer_req_body").unwrap_or(DEFAULT_BUFFER_HTTP)
        );

        let prefix_logs = parse_var(&vars, "prefix_logs").unwrap_or(DEFAULT_PREFIX_LOGS);
        let log_buffer_size = parse_var(&vars, "log_buffer_size").unwrap_or(DEFAULT_LOG_BUFFER_SIZE);

        // check
        if operational_mode == WatchdogMode::ModeHTTP && upstream_url.is_none() {
            return Err(Error::msg("For \"mode=http\" you must specify a valid URL for \"http_upstream_url\""));
        }
        if operational_mode == WatchdogMode::ModeStatic && static_path == "" {
            return Err(Error::msg("For mode=static you must specify the \"static_path\" to serve"));
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
            _wasm_root: wasm_root,
        })
    }
}


#[inline]
fn parse_var<T>(vars: &HashMap<String, String>, key: &'static str) -> Option<T> where T: FromStr {
    match vars.get(key) {
        Some(str) => {
            match str.parse::<T>() {
                Ok(res) => Some(res),
                _ => None
            }
        }
        _ => None
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::config::watchdog_config::*;
    use crate::WatchdogConfig;

    #[test]
    fn test_default() {
        let keys = vec!["fprocess", "function_process"];

        for key in keys.iter() {
            let mut env = HashMap::new();
            let f_process = "process".to_string();
            env.insert(key.to_string(), f_process.clone());

            let cfg = WatchdogConfig::new(env)
                .expect("create default watchdog config error");

            assert_eq!(cfg._tcp_port, DEFAULT_PORT);
            assert_eq!(cfg._http_read_timeout.as_secs(), DEFAULT_READ_TIMEOUT_SEC);
            assert_eq!(cfg._http_write_timeout.as_secs(), DEFAULT_WRITE_TIMEOUT_SEC);
            assert_eq!(cfg._exec_timeout.as_secs(), DEFAULT_EXEC_TIMEOUT_SEC);
            assert_eq!(cfg._health_check_interval.as_secs(), DEFAULT_WRITE_TIMEOUT_SEC);
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
            assert_eq!(cfg._wasm_root, DEFAULT_WASM_ROOT);
        }
    }

    #[test]
    fn test_empty_error() {
        let env = HashMap::new();
        let cfg = WatchdogConfig::new(env);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_static_mode() {
        let mut env = HashMap::new();
        env.insert("mode".to_string(), "static".to_string());
        let cfg = WatchdogConfig::new(env)
            .expect("create static mode watchdog config error");
        assert_eq!(cfg._function_process, String::default());
        assert_eq!(cfg._operational_mode, WatchdogMode::ModeStatic);
    }

    #[test]
    fn test_write_timeout_error() {
        let mut env = HashMap::new();
        env.insert("write_timeout".to_string(), "0".to_string());
        let cfg = WatchdogConfig::new(env);
        assert!(cfg.is_err());
    }
}
