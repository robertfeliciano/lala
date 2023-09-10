pub mod parser;
pub mod interp;

fn main() -> Result<(), anyhow::Error> {
    let raw_file = std::fs::read_to_string("matrix.lala").expect("Cannot read lala file");
    let ast_root = lala::parser::parse(&raw_file).expect("unsuccessful parse");
    println!("{:?}", &ast_root);
    interp::interp( &ast_root )?;
    Ok(())
}