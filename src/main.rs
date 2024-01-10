use std::path::Path;

pub mod interp;
pub mod parser;
pub mod repl;

gflags::define! {
    -c, --compile: &Path
}

fn main() -> Result<(), anyhow::Error> {
    let _patterns = gflags::parse();
    if COMPILE.is_present() {
        let path = COMPILE.flag.to_str().unwrap();
        let raw_file = std::fs::read_to_string(path).expect("Cannot read lala file");
        let ast_root = lala::parser::parse(&raw_file).expect("unsuccessful parse");
        interp::interp(&ast_root, None)?;
        Ok(())
    } else {
        repl::repl();
        Ok(())
    }
}
