pub mod interp;
pub mod parser;

fn main() -> Result<(), anyhow::Error> {
    let raw_file = std::fs::read_to_string("ranks.lala").expect("Cannot read lala file");
    let ast_root = lala::parser::parse(&raw_file).expect("unsuccessful parse");
    interp::interp(&ast_root)?;
    Ok(())
}
