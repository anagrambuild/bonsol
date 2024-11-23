use std::fs;

use anyhow::Result;

use crate::{command::InputSetAction, common::CliInput};

pub fn input_set(input_set: InputSetAction) -> Result<()> {
    match input_set {
        InputSetAction::Create { path } => create_input_set_file(path),
        InputSetAction::Read { path } => read_input_set_file(path),
        InputSetAction::Update { path, value } => update_input_set_file(path, value),
    }
}

pub fn create_input_set_file(path: String) -> Result<()> {
    Ok(())
}

pub fn read_input_set_file(path: String) -> Result<()> {
    Ok(())
}

pub fn update_input_set_file(path: String, value: Vec<CliInput>) -> Result<()> {
    Ok(())
}
