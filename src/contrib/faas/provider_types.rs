use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct ReplicaFuncStatus {
    pub(crate) _name: Option<String>,
    pub(crate) _image: Option<String>,
    pub(crate) _namespace: Option<String>,
    pub(crate) _env_process: Option<String>,
    pub(crate) _env_vars: Option<HashMap<String, String>>,
    pub(crate) _invocation_count: u64,
    pub(crate) _replicas: u64,
    pub(crate) _available_replicas: u64,
}

macro_rules! push_key {
    ($self:ident, $target:ident,$is_first:ident, $key:expr) => {
        if !$is_first {
            $target.push_str($self::COMMA); // ,
        }
        $target.push_str($key);
        $target.push_str($self::OBJECT_MIDDLE); // :
    };
    ($self:ident, $target:ident, $key:expr) => {
        $target.push_str($self::COMMA); // ,
        $target.push_str($key);
        $target.push_str($self::OBJECT_MIDDLE); // :
    };
}

macro_rules! push_string {
    ($self:ident, $target:ident,$is_first:ident, $key:expr,$value:expr) => {
        push_key!($self, $target, $is_first, $key);
        $target.push_str($self::STRING_QUOTATION); // "
        $self::push_escape_str(&mut $target, $value);
        $target.push_str($self::STRING_QUOTATION); // "
        $is_first = false;
    };
}

macro_rules! push_option_string {
    ($self:ident, $target:ident,$is_first:ident, $key:expr,$value:expr) => {
        if let Some(ref p) = $value {
            push_string!($self, $target, $is_first, $key, p.as_str());
        }
    };
}

impl ReplicaFuncStatus {
    const NAME_KEY: &'static str = r#""name""#;
    const IMAGE_KEY: &'static str = r#""image""#;
    const NAMESPACE_KEY: &'static str = r#""namespace""#;
    const ENV_PROCESS_KEY: &'static str = r#""envProcess""#;
    const ENV_VARS_KEY: &'static str = r#""envVars""#;
    const INVOCATION_COUNT_KEY: &'static str = r#""invocationCount""#;
    const REPLICAS_COUNT_KEY: &'static str = r#""replicas""#;
    const AVAILABLE_REPLICAS_KEY: &'static str = r#""availableReplicas""#;

    const OBJECT_LEFT: &'static str = "{";
    const OBJECT_RIGHT: &'static str = "}";
    const OBJECT_MIDDLE: &'static str = ":";
    const ARRAY_LEFT: &'static str = "[";
    const ARRAY_RIGHT: &'static str = "]";
    const COMMA: &'static str = ",";
    const STRING_QUOTATION: &'static str = "\"";

    pub(crate) fn new(replicas: u64, available_replicas: u64, invocation_count: u64) -> Self {
        Self {
            _name: None,
            _image: None,
            _namespace: None,
            _env_process: None,
            _env_vars: None,
            _invocation_count: invocation_count,
            _replicas: replicas,
            _available_replicas: available_replicas,
        }
    }

    #[inline(always)]
    fn push_escape_str(string: &mut String, s: &str) {
        let mut vec = Vec::with_capacity(s.as_bytes().len());
        s.as_bytes().iter().for_each(|c| {
            if c == &b'\n' || c == &b'\"' {
                vec.push(b'\\');
            }
            vec.push(*c);
        });
        string.push_str(std::str::from_utf8(vec.as_slice()).unwrap());
    }

    pub(crate) fn into_json(self) -> String {
        let mut json = String::new();
        json.push_str(Self::OBJECT_LEFT);
        let mut is_first = true;

        push_option_string!(Self, json, is_first, Self::NAME_KEY, self._name);
        push_option_string!(Self, json, is_first, Self::IMAGE_KEY, self._image);
        push_option_string!(Self, json, is_first, Self::NAMESPACE_KEY, self._namespace);
        push_option_string!(
            Self,
            json,
            is_first,
            Self::ENV_PROCESS_KEY,
            self._env_process
        );

        if let Some(vars) = self._env_vars {
            push_key!(Self, json, is_first, Self::ENV_VARS_KEY);
            json.push_str(Self::ARRAY_LEFT);
            let mut in_arr_is_first = true;
            vars.iter().for_each(|(k, v)| {
                push_string!(Self, json, in_arr_is_first, k.as_str(), v.as_str());
            });
            json.push_str(Self::ARRAY_RIGHT);
        }

        push_key!(Self, json, is_first, Self::REPLICAS_COUNT_KEY);
        json.push_str(self._replicas.to_string().as_str());
        push_key!(Self, json, Self::AVAILABLE_REPLICAS_KEY);
        json.push_str(self._available_replicas.to_string().as_str());
        push_key!(Self, json, Self::INVOCATION_COUNT_KEY);
        json.push_str(self._invocation_count.to_string().as_str());

        json.push_str(Self::OBJECT_RIGHT);
        json
    }
}

pub(crate) struct ScaleServiceRequest {
    pub(crate) _service_name: Option<String>,
    pub(crate) _replicas: u64,
}

impl ScaleServiceRequest {
    #[allow(dead_code)]
    const SERVICE_NAME_KEY: &'static str = r#""serviceName""#;
    const REPLICAS_KEY: &'static str = r#""replicas""#;
    const COLON: u8 = b':';

    pub(crate) fn from_json(res_s: Result<String>) -> Result<Self> {
        let s = res_s?;
        // todo: verify json string format

        let mut pos = s
            .find(Self::REPLICAS_KEY)
            .ok_or(anyhow!("Cannot find key {}", Self::REPLICAS_KEY))?;

        pos += Self::REPLICAS_KEY.as_bytes().len();

        let bytes = s.as_bytes();
        let len = bytes.len();

        // find ':'
        while pos < len {
            if bytes[pos] == Self::COLON {
                break;
            }
            pos += 1;
        }
        if pos >= len || bytes[pos] != Self::COLON {
            return Err(anyhow!("Cannot find `:` after key {}", Self::REPLICAS_KEY));
        }
        pos += 1; // ':'

        // find number
        while pos < len {
            if bytes[pos].is_ascii_digit() {
                break;
            }
            if !bytes[pos].is_ascii_whitespace() {
                return Err(anyhow!("Unexpected character ascii=`{}`", bytes[pos]));
            }
            pos += 1;
        }
        if pos >= len || !bytes[pos].is_ascii_digit() {
            return Err(anyhow!("Unexpected EOF"));
        }

        let mut replicas: u64 = 0;
        while pos < len && bytes[pos].is_ascii_digit() {
            replicas = (replicas << 3) + (replicas << 1) + ((bytes[pos] - b'0') as u64);
            pos += 1;
        }

        Ok(Self {
            _service_name: None,
            _replicas: replicas,
        })
    }
}

#[cfg(test)]
mod test {
    use super::ReplicaFuncStatus;
    use super::ScaleServiceRequest;
    use anyhow::anyhow;
    use std::collections::HashMap;

    #[test]
    fn test_to_json() {
        let replicas = 0123;
        let available_replicas = 456;
        let invoke_count = 789;
        let mut p = ReplicaFuncStatus::new(replicas, available_replicas, invoke_count);

        assert_eq!(
            p.clone().into_json(),
            format!(
                "{{{}:{},{}:{},{}:{}}}",
                ReplicaFuncStatus::REPLICAS_COUNT_KEY,
                p._replicas,
                ReplicaFuncStatus::AVAILABLE_REPLICAS_KEY,
                p._available_replicas,
                ReplicaFuncStatus::INVOCATION_COUNT_KEY,
                p._invocation_count
            )
        );

        p._name = Some("name".to_string());
        p._namespace = Some("namespace".to_string());
        let mut h = HashMap::new();
        h.insert(String::from("k1"), String::from("v1"));
        p._env_vars = Some(h);

        assert_eq!(
            p.clone().into_json(),
            format!(
                "{{{}:\"{}\",{}:\"{}\",{}:[k1:\"v1\"],{}:{},{}:{},{}:{}}}",
                ReplicaFuncStatus::NAME_KEY,
                p._name.as_ref().unwrap(),
                ReplicaFuncStatus::NAMESPACE_KEY,
                p._namespace.as_ref().unwrap(),
                ReplicaFuncStatus::ENV_VARS_KEY,
                ReplicaFuncStatus::REPLICAS_COUNT_KEY,
                p._replicas,
                ReplicaFuncStatus::AVAILABLE_REPLICAS_KEY,
                p._available_replicas,
                ReplicaFuncStatus::INVOCATION_COUNT_KEY,
                p._invocation_count
            )
        );
    }

    #[test]
    fn test_scale_service_request() {
        assert!(ScaleServiceRequest::from_json(Err(anyhow!(""))).is_err());
        assert!(ScaleServiceRequest::from_json(Ok("{{}}".to_string())).is_err());

        let str1 = format!("{{{}:123}}", ScaleServiceRequest::REPLICAS_KEY);
        assert_eq!(
            ScaleServiceRequest::from_json(Ok(str1)).unwrap()._replicas,
            123
        );

        let str2 = format!(
            "{{{} \n\t  :  \t 12366666}}",
            ScaleServiceRequest::REPLICAS_KEY
        );
        assert_eq!(
            ScaleServiceRequest::from_json(Ok(str2)).unwrap()._replicas,
            12366666
        );
    }
}
