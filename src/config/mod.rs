mod watchdog_mode;
mod watchdog_config;


use std::time::Duration;


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum WatchdogMode {
    ModeUnknown = 0,
    ModeStreaming = 1,
    ModeAfterBurn = 2,
    ModeSerializing = 3,
    ModeHTTP = 4,
    ModeStatic = 5,
    ModeWasm = 6,
}


/// configuration for a watchdog
#[derive(Debug, Clone)]
pub(crate) struct WatchdogConfig {
    /// TCP port for watchdog server
    pub(crate) _tcp_port: u16,

    pub(crate) _http_read_timeout: Duration,
    pub(crate) _http_write_timeout: Duration,
    pub(crate) _exec_timeout: Duration,
    pub(crate) _health_check_interval: Duration,

    pub(crate) _function_process: String,
    pub(crate) _content_type: String,

    pub(crate) _inject_cgi_headers: bool,
    pub(crate) _operational_mode: WatchdogMode,
    pub(crate) _suppress_lock: bool,
    pub(crate) _upstream_url: Option<String>,
    pub(crate) _static_path: String,

    /// If buffers the HTTP body in memory to prevent transfer type of chunked encoding which some servers do not support.
    pub(crate) _buffer_http_body: bool,

    /// TCP port on which to serve HTTP Prometheus metrics
    pub(crate) _metrics_port: u16,

    /// limits the number of simultaneous requests that the watchdog allows concurrently.
    /// Any request which exceeds this limit will have an immediate response of 429.
    pub(crate) _max_inflight: i32,

    /// If adds a date time stamp and the stdio name to any logging from executing functions
    pub(crate) _prefix_logs: bool,

    /// The size for scanning logs for stdout/stderr
    pub(crate) _log_buffer_size: i32,

    /// The root directory for wasm file system
    pub(crate) _wasm_root: String,
}
