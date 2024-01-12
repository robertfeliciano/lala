use std::collections::HashMap;
use super::types::*;
use super::parser;
use super::interp;

pub fn link(files: &Vec<&str>, env: &mut HashMap<String, LalaType>) -> Result<(), anyhow::Error> {
    for file in files {
        let raw_file = std::fs::read_to_string(file)?;
        let ast_root = parser::parse(&raw_file)?;
        interp::interp(&ast_root, Some(env), true)?;
    }
    Ok(())
}