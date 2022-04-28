use anyhow::{anyhow, Result};


pub(in crate::runner) fn parse_command(func: &String) -> Result<Vec<String>> {
    let v = func.split(" ")
        .map(|s| { s.to_string() })
        .collect::<Vec<String>>();

    match v.is_empty() {
        false => Ok(v),
        true => Err(anyhow!("function name cannot be empty!"))
    }
}
