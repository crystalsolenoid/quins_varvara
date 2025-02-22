use std::cmp::max;
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

    let mut mem: [u8; 0xffff] = [0; 0xffff];
    let trimmed_mem = write(&macros_applied, &mut mem);

    std::fs::write(output, trimmed_mem)?;

    Ok(())
}

fn resolve_locations<'s>(items: &'s [ROMItem]) -> HashMap<(&'s str, Option<&'s str>), u16> {
    items
        .iter()
        .scan((None, 0x0100), |(current_parent, loc), item| {
            let old_loc = *loc;
            *loc = match item {
                ROMItem::Byte(_) => *loc + 1,
                ROMItem::Location(p) => {
                    *current_parent = Some(*p);
                    *loc
                }
                ROMItem::SubLocation(_, _) => *loc,
                ROMItem::ZeroAddr(_) => *loc + 2,       // ie #01
                ROMItem::ZeroSubAddr(_, _) => *loc + 2, // ie #01
                ROMItem::Addr(_) => *loc + 3,           // ie #0104
                ROMItem::SubAddr(_, _) => *loc + 3,     // ie #0104
                ROMItem::AbsPad(a, b) => u16::from_be_bytes([*a, *b]),
                ROMItem::RelPad(a, b) => *loc + u16::from_be_bytes([*a, *b]),
                ROMItem::MacroDef(_, _) => todo!("No macros should exist at this point."),
                ROMItem::Macro(_) => todo!("No macros should exist at this point."),
            };
            Some((
                old_loc,
                match item {
                    ROMItem::SubLocation(_, c) => ROMItem::SubLocation(*current_parent, c),
                    i => i.clone(),
                },
            ))
        })
        .filter_map(|(loc, item)| match item {
            ROMItem::Location(name) => Some(((name, None), loc)),
            ROMItem::SubLocation(p, c) => Some((
                (p.expect("Sub Location found before Location!"), Some(c)),
                loc,
            )),
            _ => None,
        })
        .collect()
}

fn write<'a>(items: &[ROMItem], mem: &'a mut [u8; 0xffff]) -> &'a [u8] {
    // TODO refactor out the writing procedure. Handle wrapping address math in a
    // way that protects from zero page writes.
    let locations = resolve_locations(items);
    let mut max_written = 0x0100;
    items.iter().fold(0x0100, |i, item| match item {
        ROMItem::Byte(b) => {
            if i < 0x0100 {
                panic!("Can't write to zero page.")
            };
            mem[i as usize] = *b;
            max_written = max(max_written, i);
            i + 1
        }
        ROMItem::Location(_) => i,
        ROMItem::SubLocation(..) => i,
        ROMItem::ZeroAddr(name) => {
            if i < 0x0100 {
                panic!("Can't write to zero page.")
            };
            mem[i as usize] = 0x80;
            let [a, b] = locations[&(*name, None)].to_be_bytes();
            if a > 0 {
                panic!("Attempting to address to zero page with non-zero-page address.")
            }
            mem[i as usize + 1] = b;
            max_written = max(max_written, i + 1);
            i + 2
        }
        ROMItem::ZeroSubAddr(p, c) => {
            if i < 0x0100 {
                panic!("Can't write to zero page.")
            };
            mem[i as usize] = 0x80;
            let [a, b] = locations[&(*p, Some(*c))].to_be_bytes();
            if a > 0 {
                panic!("Attempting to address to zero page with non-zero-page address.")
            }
            mem[i as usize + 1] = b;
            max_written = max(max_written, i + 1);
            i + 2
        }
        ROMItem::Addr(name) => {
            if i < 0x0100 {
                panic!("Can't write to zero page.")
            };
            mem[i as usize] = 0xa0;
            let [a, b] = locations[&(*name, None)].to_be_bytes();
            mem[i as usize + 1] = a;
            mem[i as usize + 2] = b;
            max_written = max(max_written, i + 2);
            i + 3
        }
        ROMItem::SubAddr(parent, child) => {
            if i < 0x0100 {
                panic!("Can't write to zero page.")
            };
            mem[i as usize] = 0xa0;
            let [a, b] = locations[&(*parent, Some(*child))].to_be_bytes();
            mem[i as usize + 1] = a;
            mem[i as usize + 2] = b;
            max_written = max(max_written, i + 2);
            i + 3
        }
        ROMItem::AbsPad(a, b) => u16::from_be_bytes([*a, *b]),
        ROMItem::RelPad(a, b) => i + u16::from_be_bytes([*a, *b]),
        ROMItem::MacroDef(_, _) => panic!("No macros should exist at this point."),
        ROMItem::Macro(_) => panic!("No macros should exist at this point."),
    });
    &mem[0x0100..=max_written as usize]
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
        let desired = HashMap::from([(("test", None), 0x0101)]);
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

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0xa0, 0x01, 0x04, 0x00, 0xff];
        assert_eq!(trimmed_mem, desired);
    }

    #[test]
    fn pad_location_label() {
        // |4001 @label |0100 ;label
        let items = vec![
            ROMItem::AbsPad(0x40, 0x01),
            ROMItem::Location("label"),
            ROMItem::AbsPad(0x01, 0x00),
            ROMItem::Addr("label"),
        ];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0xa0, 0x40, 0x01];
        assert_eq!(trimmed_mem, desired);
    }

    #[test]
    fn pad_location_write() {
        // |0102 @label ;label
        let items = vec![
            ROMItem::AbsPad(0x01, 0x02),
            ROMItem::Location("label"),
            ROMItem::Addr("label"),
        ];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0x00, 0x00, 0xa0, 0x01, 0x02];
        assert_eq!(trimmed_mem, desired);
    }

    #[test]
    #[should_panic]
    fn no_zero_page_write() {
        // |00 10
        let items = vec![ROMItem::AbsPad(0x00, 0x00), ROMItem::Byte(10)];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let _trimmed_mem = write(&items, &mut mem);
    }

    #[test]
    fn rel_pad() {
        // |0102 $02 @label ;label
        let items = vec![
            ROMItem::AbsPad(0x01, 0x02),
            ROMItem::RelPad(0x00, 0x02),
            ROMItem::Location("label"),
            ROMItem::Addr("label"),
        ];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0x00, 0x00, 0x00, 0x00, 0xa0, 0x01, 0x04];
        assert_eq!(trimmed_mem, desired);
    }

    #[test]
    fn sub_label() {
        // |00 @label $2 &a
        // |0100 ;label/a
        let items = vec![
            ROMItem::AbsPad(0x00, 0x00),
            ROMItem::Location("label"),
            ROMItem::RelPad(0x00, 0x02),
            ROMItem::SubLocation(None, "a"),
            ROMItem::AbsPad(0x01, 0x00),
            ROMItem::SubAddr("label", "a"),
        ];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0xa0, 0x00, 0x02];
    }

    #[test]
    fn zero_page_addr() {
        // |10 @label
        // |0100 .label
        let items = vec![
            ROMItem::AbsPad(0x00, 0x10),
            ROMItem::Location("label"),
            ROMItem::AbsPad(0x01, 0x00),
            ROMItem::ZeroAddr("label"),
        ];

        let mut mem: [u8; 0xffff] = [0; 0xffff];
        let trimmed_mem = write(&items, &mut mem);

        let desired = vec![0x80, 0x10];
    }
}
