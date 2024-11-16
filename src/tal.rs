use std::fs::File;
use std::io::prelude::*;

use crate::parse::{parse_tal, ROMItem};
use winnow::Parser;

pub fn assemble(input: &str, output: &str) -> std::io::Result<()> {
    let mut input = File::open(input)?;

    let mut contents = String::new();
    input.read_to_string(&mut contents)?;

    let hex: Vec<u8> = parse_tal
        .parse(&contents)
        .unwrap()
        .into_iter()
        .filter_map(|item| match item {
            ROMItem::Location(_) => None,
            ROMItem::Byte(b) => Some(b),
        })
        .collect();

    std::fs::write(output, &hex)?;

    Ok(())
}
