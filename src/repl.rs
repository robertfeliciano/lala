use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};

use interp::interp;
use lala::types::*;

use crate::interp;

pub fn repl() -> Result<(), anyhow::Error> {
    let mut env: HashMap<String, LalaType> = HashMap::new();
    println!("Lala Shell v0.0.1");
    loop {
        print!("\x1b[1mλ \x1b[0m");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let line = input.trim();
        let ast = lala::parser::parse(line)?;
        interp(&ast, Some(&mut env))?;
    }
}
