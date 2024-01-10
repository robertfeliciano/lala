use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};

use interp::interp;
use lala::types::*;

use crate::interp;

pub fn repl() {
    let mut env: HashMap<String, LalaType> = HashMap::new();
    loop {
        print!("> ");
        let _ = stdout().flush();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let line = input.trim();
        let ast = lala::parser::parse(line).expect("unsuccessful parse");
        // do question mark thing
        let _ = interp(&ast, Some(&mut env));
    }
}
