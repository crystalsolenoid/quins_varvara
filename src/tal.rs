use std::fs::File;
use std::io::prelude::*;

use crate::parse::parse_tal;
use winnow::Parser;

pub fn assemble(input: &str, output: &str) -> std::io::Result<()> {
    let mut input = File::open(input)?;

    let mut contents = String::new();
    input.read_to_string(&mut contents)?;

    let hex: Vec<u8> = parse_tal.parse(&contents).unwrap();

    std::fs::write(output, &hex)?;

    Ok(())
}
