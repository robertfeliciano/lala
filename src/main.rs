use std::path::Path;

use anyhow::anyhow;

mod interp;
mod linalg;
mod parser;
mod repl;
mod types;

gflags::define! {
    -c, --compile: &Path
}

fn get_extension(path: &str) -> Result<&str, anyhow::Error> {
    let ext = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap();
    if ext != "lala" {
        Err(anyhow!("Not a lala file!"))
    } else {
        Ok(ext)
    }
}

fn main() -> Result<(), anyhow::Error> {
    let _patterns = gflags::parse();
    if COMPILE.is_present() {
        let path = COMPILE.flag.to_str().unwrap();
        match get_extension(path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                return Ok(());
            }
        }

        let raw_file = std::fs::read_to_string(path)?;

        let ast_root = parser::parse(&raw_file)?;
        let res = interp::interp(&ast_root, None, false)?;
        println!("{}", res);
        Ok(())
    } else {
        // repl::repl()
        Ok(())
    }
}
