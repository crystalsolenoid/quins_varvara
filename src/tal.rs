use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::parse::{parse_tal, ROMItem};
use winnow::Parser;

pub fn assemble(input: &str, output: &str) -> std::io::Result<()> {
    let mut input = File::open(input)?;

    let mut contents = String::new();
    input.read_to_string(&mut contents)?;

    let parsed: Vec<ROMItem> = parse_tal.parse(&contents).unwrap();

    let hex: Vec<u8> = parsed
        .into_iter()
        .filter_map(|item| match item {
            ROMItem::Location(_) => None,
            ROMItem::Addr(_) => todo!(),
            ROMItem::Byte(b) => Some(b),
        })
        .collect();

    std::fs::write(output, &hex)?;

    Ok(())
}

fn resolve_locations<'s>(items: &'s [ROMItem]) -> HashMap<&'s str, u16> {
    items
        .into_iter()
        .scan(0x0100, |state, item| {
            let old_state = *state;
            *state += match item {
                ROMItem::Byte(_) => 1,
                ROMItem::Location(_) => 0,
                ROMItem::Addr(_) => 3, // ie #0104
            };
            Some((old_state, item))
        })
        .filter_map(|(loc, item)| match item {
            ROMItem::Location(name) => Some((*name, loc)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_one_location() {
        let items = vec![ROMItem::Byte(0x00), ROMItem::Location("test")];
        let locations = resolve_locations(&items);
        let desired = HashMap::from([("test", 0x0101)]);
        assert_eq!(locations, desired);
    }
}
