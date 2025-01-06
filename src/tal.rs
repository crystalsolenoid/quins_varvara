use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::parse::{parse_tal, ROMItem, State, Stream};
use winnow::Parser;

pub fn assemble(input: &str, output: &str) -> std::io::Result<()> {
    let mut input = File::open(input)?;

    let mut contents = String::new();
    input.read_to_string(&mut contents)?;

    let state = HashMap::new();
    let stream = Stream {
        input: &contents,
        state: State(state),
    };
    let parsed: Vec<ROMItem> = parse_tal.parse(stream).unwrap();

    let macros_applied = apply_macros(&parsed);

    let hex = replace_locations(&macros_applied);

    std::fs::write(output, &hex)?;

    Ok(())
}

fn resolve_locations<'s>(items: &'s [ROMItem]) -> HashMap<&'s str, u16> {
    items
        .iter()
        .scan(0x0100, |state, item| {
            let old_state = *state;
            *state += match item {
                ROMItem::Byte(_) => 1,
                ROMItem::Location(_) => 0,
                ROMItem::Addr(_) => 3, // ie #0104
                ROMItem::MacroDef(_, _) => todo!("No macros should exist at this point."),
                ROMItem::Macro(_) => todo!("No macros should exist at this point."),
            };
            Some((old_state, item))
        })
        .filter_map(|(loc, item)| match item {
            ROMItem::Location(name) => Some((*name, loc)),
            _ => None,
        })
        .collect()
}

fn replace_locations(items: &[ROMItem]) -> Vec<u8> {
    let locations = resolve_locations(items);
    items
        .iter()
        .filter_map(|item| match item {
            ROMItem::Byte(b) => Some(vec![*b]),
            ROMItem::Location(_) => None,
            ROMItem::Addr(name) => {
                let mut bytes = vec![0xa0];
                bytes.extend(locations[name].to_be_bytes());
                Some(bytes)
            }
            ROMItem::MacroDef(_, _) => todo!("No macros should exist at this point."),
            ROMItem::Macro(_) => todo!("No macros should exist at this point."),
        })
        .flatten()
        .collect()
}

fn apply_macros<'s>(items: &'s [ROMItem]) -> Vec<ROMItem<'s>> {
    let mut defined_macros: HashMap<&str, &Vec<ROMItem>> = HashMap::new();
    items
        .iter()
        .filter_map(|item| {
            let answer: Option<Vec<ROMItem<'_>>> = match item {
                ROMItem::MacroDef(name, contents) => {
                    defined_macros.insert(name, contents);
                    None
                }
                ROMItem::Macro(name) => Some(defined_macros.get(name).unwrap().to_vec()), // TODO fix clones?
                other => Some(vec![other.clone()]), // TODO clone
            };
            answer
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn macros() {
        let in_macro = " #0008 ";
        let state = HashMap::new();
        let stream = Stream {
            input: &in_macro,
            state: State(state),
        };

        let out_macro: Vec<ROMItem> = parse_tal.parse(stream).unwrap();
        let parsed: Vec<ROMItem> = vec![
            ROMItem::MacroDef("INIT-X", out_macro),
            ROMItem::Macro("INIT-X"),
        ];

        let macros_applied = apply_macros(&parsed);

        assert_eq!(
            macros_applied,
            vec![
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0x00),
                ROMItem::Byte(0x08)
            ]
        );
    }

    #[test]
    fn read_one_location() {
        let items = vec![ROMItem::Byte(0x00), ROMItem::Location("test")];
        let locations = resolve_locations(&items);
        let desired = HashMap::from([("test", 0x0101)]);
        assert_eq!(locations, desired);
    }

    #[test]
    fn replace_location() {
        let items = vec![
            ROMItem::Addr("arrow"),
            ROMItem::Byte(0x00),
            ROMItem::Location("arrow"),
            ROMItem::Byte(0xff),
        ];
        let desired = vec![0xa0, 0x01, 0x04, 0x00, 0xff];
        let replaced_items = replace_locations(&items);
        assert_eq!(replaced_items, desired);
    }
}
