use std::collections::HashMap;
use std::env;

use anyhow::{anyhow, Result};
use hyper::{Body, Request};
use lazy_static::lazy_static;

lazy_static! {
    // skip the no UTF-8 env var
    static ref ENVIRONMENT_VARS : HashMap<String,String> = env::vars_os().filter_map(|(k_os, v_os)| {
        match (k_os.into_string(), v_os.into_string()) {
            (Ok(k), Ok(v)) => Some((k, v)),
            _ => None
        }
    }).collect();
}

#[inline(always)]
pub(crate) fn parse_command(func: &String) -> Result<Vec<String>> {
    let v = func
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    match v.is_empty() {
        false => Ok(v),
        true => Err(anyhow!("function name cannot be empty!")),
    }
}

#[inline(always)]
pub(crate) fn environment_vars() -> &'static HashMap<String, String> {
    &ENVIRONMENT_VARS
}

#[inline(always)]
pub(crate) fn inject_environment(inherit: bool, req: &Request<Body>) -> HashMap<String, String> {
    let mut res = if inherit {
        ENVIRONMENT_VARS.clone()
    } else {
        HashMap::new()
    };

    for (k, v) in req.headers().iter() {
        if let Ok(val) = v.to_str() {
            let key = format!("Http_{}", k.to_string().replace('-', "_"));
            res.insert(key, val.to_string());
        }
    }

    res.insert("Http_Path".to_string(), req.uri().path().to_string());
    res.insert("Http_Method".to_string(), req.method().to_string());
    if let Some(q) = req.uri().query() {
        res.insert("Http_Query".to_string(), q.to_string());
    }
    // todo: Http_Transfer_Encoding

    res
}

macro_rules! env_get_or_warn {
    ($cfg:expr,$key:expr,$default:expr) => {
        match $cfg {
            None => {
                log::warn!(
                    "The environment variable `{}` is not specified, use the default value: `{}`",
                    $key,
                    $default
                );
                $default
            }
            Some(v) => {
                log::info!("Set {} = `{}`", $key, v);
                v
            }
        }
    };
}
