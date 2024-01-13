use super::interp;
use super::parser;
use super::types::*;
use std::collections::HashMap;

pub fn link(files: &Vec<&str>, env: &mut HashMap<String, LalaType>) -> Result<(), anyhow::Error> {
    for file in files {
        let raw_file = std::fs::read_to_string(file)?;
        let ast_root = parser::parse(&raw_file)?;
        // IDEA: pass in file name option for linkage instead of bool, append it to the beginning of the linked var names
        interp::interp(&ast_root, Some(env), true)?;
    }
    Ok(())
}

pub fn debug(env: &HashMap<String, LalaType>) -> Result<(), anyhow::Error> {
    println!("{:?}", env);
    Ok(())
}
