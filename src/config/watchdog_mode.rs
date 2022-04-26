use core::fmt::Display;
use std::fmt::Formatter;
use crate::config::WatchdogMode;


impl From<usize> for WatchdogMode {
    fn from(num: usize) -> Self {
        match num {
            0 => WatchdogMode::ModeUnknown,
            1 => WatchdogMode::ModeStreaming,
            2 => WatchdogMode::ModeAfterBurn,
            3 => WatchdogMode::ModeSerializing,
            4 => WatchdogMode::ModeHTTP,
            5 => WatchdogMode::ModeStatic,
            6 => WatchdogMode::ModeWasm,
            _ => WatchdogMode::ModeUnknown,
        }
    }
}


const WATCHDOG_MODE_STR: [&str; 7] = ["unknown", "streaming", "afterburn",
    "serializing", "http", "static", "wasm"];


impl From<&str> for WatchdogMode {
    fn from(str: &str) -> Self {
        for (i, s) in WATCHDOG_MODE_STR.iter().enumerate() {
            if str == *s {
                return WatchdogMode::try_from(i).unwrap();
            }
        }
        WatchdogMode::ModeUnknown
    }
}


impl From<String> for WatchdogMode {
    fn from(s: String) -> Self {
        WatchdogMode::from(s.as_str())
    }
}


impl From<&String> for WatchdogMode {
    fn from(s: &String) -> Self {
        WatchdogMode::from(s.as_str())
    }
}


impl From<WatchdogMode> for String {
    fn from(mode: WatchdogMode) -> Self {
        WATCHDOG_MODE_STR[mode as usize].to_string()
    }
}


impl Display for WatchdogMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(*self).as_str())
    }
}


#[cfg(test)]
mod test {
    use crate::config::watchdog_mode::{WATCHDOG_MODE_STR, WatchdogMode};

    #[test]
    fn test_mode() {
        for str in WATCHDOG_MODE_STR.iter() {
            let mode = WatchdogMode::from(str.to_string());
            assert_eq!(String::from(mode).as_str(), *str);
        }
    }
}
