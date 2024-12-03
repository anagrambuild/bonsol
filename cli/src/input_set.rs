use std::{
    fs,
    io::{Read, Write},
    path,
};

use anyhow::{bail, Result};

use crate::{command::InputSetAction, common::CliInput};

pub fn input_set(action: InputSetAction) -> Result<()> {
    match action {
        InputSetAction::Create {
            path,
            inputs,
            truncate,
        } => create_input_set_file(
            || try_get_file_contents(&path, true),
            &path,
            inputs,
            truncate,
        ),
        InputSetAction::Read { path } => {
            read_input_set_file(|| try_get_file_contents(&path, false))
        }
        InputSetAction::Update { path, inputs } => {
            update_input_set_file(|| try_get_file_contents(&path, false), &path, inputs)
        }
    }
}

fn try_get_file_contents(path: &str, create: bool) -> Result<Vec<u8>> {
    if !path::Path::new(path).exists() && !create {
        bail!("File path '{path}' does not exist.\nPlease check that the given path is correct and try again.")
    }

    let mut contents = Vec::new();
    let Ok(mut file) = fs::File::open(path) else {
        return Ok(contents);
    };
    file.read_to_end(&mut contents)?;

    Ok(contents)
}

fn new_input_set() -> serde_json::Value {
    serde_json::json!({ "inputs": [] })
}

fn get_input_set(contents: &[u8], truncate: bool) -> Result<serde_json::Value> {
    if !contents.is_empty() && !truncate {
        return Ok(serde_json::from_slice(&contents)?);
    }

    Ok(new_input_set())
}

fn extend_input_set(json_value: &mut serde_json::Value, inputs: &[CliInput]) {
    let inputs_array = json_value
        .get_mut("inputs")
        .and_then(serde_json::Value::as_array_mut)
        .expect("An 'inputs' key is missing from the json object, ie. '{ \"inputs\": [] }'");
    inputs_array.extend(
        inputs
            .iter()
            .filter_map(|input| match serde_json::to_value(input) {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("The following input is malformed: {input:?}\n{e}");
                    None
                }
            }),
    );
}

pub(crate) fn input_set_with_opts(
    try_get_contents: impl FnOnce() -> Result<Vec<u8>>,
    inputs: Option<Vec<CliInput>>,
    truncate: bool,
) -> Result<serde_json::Value> {
    let contents = try_get_contents()?;

    let mut json_value = get_input_set(&contents, truncate)?;
    inputs
        .as_deref()
        .map(|inputs| extend_input_set(&mut json_value, inputs));

    Ok(json_value)
}

pub fn create_input_set_file(
    try_get_contents: impl FnOnce() -> Result<Vec<u8>>,
    path: &str,
    inputs: Option<Vec<CliInput>>,
    truncate: bool,
) -> Result<()> {
    let json_value = input_set_with_opts(try_get_contents, inputs, truncate)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(serde_json::to_string_pretty(&json_value)?.as_bytes())?;

    Ok(())
}

pub fn read_input_set_file(try_get_contents: impl FnOnce() -> Result<Vec<u8>>) -> Result<()> {
    let contents = try_get_contents()?;
    let json_value: serde_json::Value = serde_json::from_slice(&contents)?;
    let json_value_pretty = serde_json::to_string_pretty(&json_value)?;
    println!("{json_value_pretty}");

    Ok(())
}

pub fn update_input_set_file(
    try_get_contents: impl FnOnce() -> Result<Vec<u8>>,
    path: &str,
    inputs: Vec<CliInput>,
) -> Result<()> {
    let json_value = input_set_with_opts(try_get_contents, Some(inputs), false)?;
    let mut file = fs::OpenOptions::new()
        .create(false)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(serde_json::to_string_pretty(&json_value)?.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::input_set_with_opts;
    use crate::common::CliInput;

    const DEFAULT_TEST_EMPTY_SET: &[u8; 16] = br#"{ "inputs": [] }"#;
    const DEFAULT_TEST_INPUT_SET: &[u8; 447]= br#"{
            "inputs": [
                {
                  "inputType": "PublicData",
                  "data": "{\"attestation\":\"test\"}"
                },
                {
                  "inputType": "Private",
                  "data": "https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA"
                }
            ]
        }"#;

    fn test_inputs() -> Vec<CliInput> {
        vec![CliInput {
            input_type: "PublicData".into(),
            data: "test".into(),
        }]
    }

    #[test]
    fn test_create() {
        let set = input_set_with_opts(
            || Ok(DEFAULT_TEST_EMPTY_SET.to_vec()),
            Some(test_inputs()),
            false,
        )
        .unwrap();
        let inputs = set
            .get("inputs")
            .and_then(serde_json::Value::as_array)
            .unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0]["inputType"].as_str(), Some("PublicData"));
        assert_eq!(inputs[0]["data"].as_str(), Some("test"));
    }

    #[test]
    fn test_truncate() {
        let set = input_set_with_opts(
            || Ok(DEFAULT_TEST_INPUT_SET.to_vec()),
            Some(test_inputs()),
            true,
        )
        .unwrap();
        let inputs = set
            .get("inputs")
            .and_then(serde_json::Value::as_array)
            .unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0]["inputType"].as_str(), Some("PublicData"));
        assert_eq!(inputs[0]["data"].as_str(), Some("test"));
    }

    #[test]
    fn test_append() {
        let set = input_set_with_opts(
            || Ok(DEFAULT_TEST_INPUT_SET.to_vec()),
            Some(test_inputs()),
            false,
        )
        .unwrap();
        let inputs = set
            .get("inputs")
            .and_then(serde_json::Value::as_array)
            .unwrap();

        assert_eq!(inputs.len(), 3);
        assert_eq!(inputs[2]["inputType"].as_str(), Some("PublicData"));
        assert_eq!(inputs[2]["data"].as_str(), Some("test"));
    }
}
