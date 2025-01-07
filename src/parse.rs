use std::collections::HashMap;
use winnow::combinator::{alt, delimited, dispatch, fail, repeat, separated};
use winnow::stream::AsChar;
use winnow::token::{any, one_of, take_till, take_until, take_while};
use winnow::{PResult, Parser, Stateful};

use crate::opcode::{encode_base_code, BASE_OPCODES};

#[derive(Debug, PartialEq, Clone)]
pub enum ROMItem<'s> {
    Byte(u8),
    Location(&'s str),
    Addr(&'s str),
    MacroDef(&'s str, Vec<ROMItem<'s>>),
    Macro(&'s str),
    AbsPad(u8, u8),
    RelPad(u8, u8),
}

#[derive(Debug)]
pub struct State<'s>(pub HashMap<&'s str, Vec<ROMItem<'s>>>);

impl<'s> State<'s> {
    fn define(&mut self, name: &'s str, content: Vec<ROMItem<'s>>) {
        self.0.insert(name, content);
    }
}

pub type Stream<'is> = Stateful<&'is str, State<'is>>;

fn parse_macro_def<'s>(i: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let name = take_label.parse_next(i)?;
    take_whitespace0.parse_next(i)?;
    let cont = delimited('{', parse_tal, '}').parse_next(i)?;
    let macro_def = vec![ROMItem::MacroDef(name, cont)];
    match macro_def.clone()[0] {
        // todo remove clone
        ROMItem::MacroDef(_, ref c) => i.state.define(name, c.to_vec()),
        _ => panic!("should always be a MacroDef"),
    };
    Ok(macro_def)
}

// from stash
/*
fn macro_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let name = take_label(input)?;
    let contents = delimited('{', parse_tal, '}').parse_next(input)?;

    Ok(vec![ROMItem::MacroDef(name, contents)])
}
*/

// from stash
fn parse_macro_call<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let name = take_label.parse_next(input)?;
    if input.state.0.contains_key(name) {
        Ok(vec![ROMItem::Macro(name)])
    } else {
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

pub fn parse_tal<'s>(stream: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    take_whitespace0.parse_next(stream)?;
    let bytes: Vec<Vec<ROMItem>> =
        separated(0.., next_tokens, take_whitespace1).parse_next(stream)?;
    take_whitespace0.parse_next(stream)?;
    Ok(bytes.into_iter().flatten().collect())
}

fn next_tokens<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let out = alt((
        parse_comment,
        // todo should I allow macro definitions within macros? maybe implies
        // lies about the presence scope (there is none)
        parse_rune,
        parse_opcode,
        parse_many_hexbytes,
        parse_macro_call,
    ))
    .parse_next(input)?;
    Ok(out)
}

fn parse_todo<'s>(_input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    todo!();
}

fn parse_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    dispatch! {any;
        '%' => parse_macro_def,
        '|' => abs_pad_rune,
        '$' => rel_pad_rune,
        '@' => label_rune,
        '&' => sublabel_rune,
        '#' => lit_rune,
        '.' => parse_todo,
        ',' => parse_todo,
        ';' => abs_addr_rune,
        ':' => parse_todo,
        '\'' => parse_todo,
        '"' => parse_todo,
        _ => fail::<_, Vec<ROMItem>, _>,
    }
    .parse_next(input)
}

fn sublabel_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    todo!()
}

fn lit_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    alt((lit_rune_short, lit_rune_byte)).parse_next(input)
}

fn lit_rune_byte<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let byte = parse_hexbyte.parse_next(input)?;
    Ok(vec![ROMItem::Byte(0x80), byte])
}

fn lit_rune_short<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let short = parse_hexshort.parse_next(input)?;
    Ok(vec![ROMItem::Byte(0xa0), short.0, short.1])
}

fn label_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let label = take_label(input)?;
    Ok(vec![ROMItem::Location(label)])
}

fn abs_addr_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let label = take_label(input)?;
    Ok(vec![ROMItem::Addr(label)])
}

fn rel_pad_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr =
        alt((rel_pad_rune_short, rel_pad_rune_byte, rel_pad_rune_nibble)).parse_next(input)?;
    Ok(addr)
}

fn rel_pad_rune_nibble<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = parse_nibble.parse_next(input)?;
    Ok(vec![ROMItem::RelPad(0, addr)])
}

fn rel_pad_rune_byte<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = parse_hexbyte.parse_next(input)?;
    match addr {
        ROMItem::Byte(b) => Ok(vec![ROMItem::RelPad(0, b)]),
        _ => panic!("parse_hexbyte Should always output Byte"),
    }
}

fn rel_pad_rune_short<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = parse_hexshort.parse_next(input)?;
    match addr {
        (ROMItem::Byte(b), ROMItem::Byte(c)) => Ok(vec![ROMItem::RelPad(b, c)]),
        _ => panic!("parse_hexshort Should always output a tuple of Bytes"),
    }
}

fn abs_pad_rune<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = alt((abs_pad_rune_short, abs_pad_rune_byte)).parse_next(input)?;
    Ok(addr)
}

fn abs_pad_rune_byte<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = parse_hexbyte.parse_next(input)?;
    match addr {
        ROMItem::Byte(b) => Ok(vec![ROMItem::AbsPad(0, b)]),
        _ => panic!("parse_hexbyte Should always output Byte"),
    }
}

fn abs_pad_rune_short<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let addr = parse_hexshort.parse_next(input)?;
    match addr {
        (ROMItem::Byte(b), ROMItem::Byte(c)) => Ok(vec![ROMItem::AbsPad(b, c)]),
        _ => panic!("parse_hexshort Should always output a tuple of Bytes"),
    }
}

fn take_label<'s>(input: &mut Stream<'s>) -> PResult<&'s str> {
    take_till(1.., (AsChar::is_space, AsChar::is_newline)).parse_next(input)
}

fn take_whitespace1<'s>(input: &mut Stream<'s>) -> PResult<&'s str> {
    take_while(1.., (AsChar::is_space, AsChar::is_newline, '[', ']')).parse_next(input)
}

fn take_whitespace0<'s>(input: &mut Stream<'s>) -> PResult<&'s str> {
    take_while(0.., (AsChar::is_space, AsChar::is_newline, '[', ']')).parse_next(input)
}

fn parse_comment<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    ('(', take_until(0.., ')'), ')').parse_next(input)?;
    Ok(vec![])
}

fn parse_base_opcode<'s>(input: &mut Stream<'s>) -> PResult<&'s str> {
    alt(BASE_OPCODES).parse_next(input)
}

fn calculate_base_opcode(input: &mut Stream<'_>) -> PResult<u8> {
    parse_base_opcode
        .map(|s: &str| encode_base_code(s))
        .parse_next(input)
}

fn calculate_flags(input: &mut Stream<'_>) -> PResult<u8> {
    let flags = parse_opcode_flags.parse_next(input)?;
    let mut byte = 0;
    if flags.0 {
        byte |= 0b001_00000;
    }
    if flags.1 {
        byte |= 0b100_00000;
    }
    if flags.2 {
        byte |= 0b010_00000;
    }
    Ok(byte)
}

fn parse_opcode_flags(input: &mut Stream<'_>) -> PResult<(bool, bool, bool)> {
    (parse_short_flag, parse_keep_flag, parse_return_flag).parse_next(input)
}

fn parse_short_flag(input: &mut Stream<'_>) -> PResult<bool> {
    let flag = repeat(0..=1, "2").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_return_flag(input: &mut Stream<'_>) -> PResult<bool> {
    let flag = repeat(0..=1, "r").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_keep_flag(input: &mut Stream<'_>) -> PResult<bool> {
    let flag = repeat(0..=1, "k").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_opcode<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    let (base, flags) = (calculate_base_opcode, calculate_flags).parse_next(input)?;
    // if base & flags != 0, that's when we return an error
    // because it means an invalid flag has been used
    // ie LITk
    if base & flags != 0 {
        return fail(input);
    }
    Ok(vec![ROMItem::Byte(base | flags)])
}

fn hex_digit_to_u8(input: char) -> u8 {
    match input {
        '0' => 0x0,
        '1' => 0x1,
        '2' => 0x2,
        '3' => 0x3,
        '4' => 0x4,
        '5' => 0x5,
        '6' => 0x6,
        '7' => 0x7,
        '8' => 0x8,
        '9' => 0x9,
        'a' => 0xa,
        'b' => 0xb,
        'c' => 0xc,
        'd' => 0xd,
        'e' => 0xe,
        'f' => 0xf,
        _ => panic!(), // TODO error better
    }
}

const HEX_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

// TODO use an "in sequence" combinator?
fn parse_nibble<'s>(input: &mut Stream<'s>) -> PResult<u8> {
    let nibble = one_of(HEX_DIGITS).parse_next(input)?;
    let byte = hex_digit_to_u8(nibble);
    Ok(byte)
}

// TODO use an "in sequence" combinator?
fn parse_hexbyte<'s>(input: &mut Stream<'s>) -> PResult<ROMItem<'s>> {
    let high = one_of(HEX_DIGITS).parse_next(input)?;
    let low = one_of(HEX_DIGITS).parse_next(input)?;
    let byte = (hex_digit_to_u8(high) << 4) + hex_digit_to_u8(low);
    Ok(ROMItem::Byte(byte))
}

fn parse_many_hexbytes<'s>(input: &mut Stream<'s>) -> PResult<Vec<ROMItem<'s>>> {
    repeat(1.., parse_hexbyte).parse_next(input)
}

// TODO use an "in sequence" combinator?
fn parse_hexshort<'s>(input: &mut Stream<'s>) -> PResult<(ROMItem<'s>, ROMItem<'s>)> {
    let high = parse_hexbyte.parse_next(input)?;
    let low = parse_hexbyte.parse_next(input)?;
    Ok((high, low))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hexbyte() {
        let input = "fd .System/r";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_hexbyte.parse_next(&mut stream).unwrap();

        assert_eq!(output, ROMItem::Byte(0xfd));

        assert!(parse_hexbyte.parse_next(&mut stream).is_err());
    }

    #[test]
    fn hexshort() {
        let input = "4cfd .System/r";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_hexshort.parse_next(&mut stream).unwrap();

        assert_eq!(output, (ROMItem::Byte(0x4c), ROMItem::Byte(0xfd)));
    }

    #[test]
    fn single_digit_isnt_hexbyte() {
        let input = "d .System/r";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_hexbyte.parse_next(&mut stream);

        assert!(output.is_err());
    }

    #[test]
    fn parses_opcode() {
        let input = "SUB2 ;on-frame";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_opcode.parse_next(&mut stream).unwrap();

        assert_eq!(output, vec!(ROMItem::Byte(0x39)));
    }

    #[test]
    fn parses_flags() {
        let input = "2k ;on-frame";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_opcode_flags.parse_next(&mut stream).unwrap();

        assert_eq!(output, (true, true, false));
    }

    #[test]
    fn parses_base_opcode_byte() {
        let input = "INC ;on-frame";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = calculate_base_opcode.parse_next(&mut stream).unwrap();

        assert_eq!(output, 0x01);
    }

    #[test]
    fn fails_on_opcode_then_nonsense() {
        let input = "SUB2abc ";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream);

        assert!(output.is_err());
    }

    #[test]
    fn parses_base_opcode() {
        let input = "SUB2 ;on-frame";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_base_opcode.parse_next(&mut stream).unwrap();

        assert_eq!(output, "SUB");
    }

    #[test]
    fn errs_on_litk() {
        let input = "LITk 1234";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output = parse_opcode.parse_next(&mut stream);
        assert!(output.is_err());
    }

    #[test]
    fn comment() {
        let input = "( Comment ) SUB2 INC";
        let state = State(HashMap::new());
        let mut stream = Stream { input, state };

        let output_1 = parse_comment.parse_next(&mut stream).unwrap();
        dbg!(&stream);
        let _ = take_whitespace1.parse_next(&mut stream);
        let output_2 = parse_opcode.parse_next(&mut stream).unwrap();

        assert_eq!(output_1, vec![]);
        assert_eq!(output_2, vec![ROMItem::Byte(0x39)]);
    }

    #[test]
    fn parses_bytes_only_space_delimited() {
        let input = "a0 ff 80";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();
        assert_eq!(
            output,
            vec!(
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0xff),
                ROMItem::Byte(0x80)
            )
        );
    }

    #[test]
    fn parse_unseparated_bytes() {
        let input = "a0ff80";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_many_hexbytes.parse(stream).unwrap();

        assert_eq!(
            output,
            vec!(
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0xff),
                ROMItem::Byte(0x80)
            )
        );
    }

    #[test]
    fn parses_bytes() {
        let input = "a0 ff80";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();
        assert_eq!(
            output,
            vec!(
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0xff),
                ROMItem::Byte(0x80)
            )
        );
    }

    #[test]
    fn parses_bytes_and_opcodes() {
        let input = "a0 BRK ff80 LIT";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();
        assert_eq!(
            output,
            vec!(
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0x00),
                ROMItem::Byte(0xff),
                ROMItem::Byte(0x80),
                ROMItem::Byte(0x80)
            )
        );
    }

    #[test]
    fn ignores_comment() {
        let input = "a0 BRK (test comment!) ff80 LIT";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(
            output,
            vec!(
                ROMItem::Byte(0xa0),
                ROMItem::Byte(0x00),
                ROMItem::Byte(0xff),
                ROMItem::Byte(0x80),
                ROMItem::Byte(0x80)
            )
        );
    }

    #[test]
    fn errs_when_opcode_has_no_whitespace_before() {
        let input = "a0BRK ff80 LIT";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream);

        assert!(output.is_err());
    }

    #[test]
    fn errs_when_opcode_has_no_whitespace_after() {
        let input = "a0 BRKff80 LIT";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream);

        assert!(output.is_err());
    }

    #[test]
    fn lit_rune() {
        let input = "#10";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(output, vec!(ROMItem::Byte(0x80), ROMItem::Byte(0x10)));
    }

    #[test]
    fn parses_multiline() {
        let input = "#2ce9 #08 DEO2\n";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream);

        assert!(output.is_ok());
    }

    #[test]
    fn label_rune() {
        let input = "@test ";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output_rune = parse_tal.parse(stream).unwrap();

        assert_eq!(output_rune, vec![ROMItem::Location("test")]);
    }

    #[test]
    fn absolute_addr_rune() {
        let input = ";test ";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output_rune = parse_tal.parse(stream).unwrap();

        assert_eq!(output_rune, vec![ROMItem::Addr("test")]);
    }

    #[test]
    fn basic_macro() {
        let input = "%EMIT { #00 }";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output_rune = parse_tal.parse(stream).unwrap();
        assert_eq!(
            output_rune,
            vec![ROMItem::MacroDef(
                "EMIT",
                vec![ROMItem::Byte(0x80), ROMItem::Byte(0x00)]
            )]
        );
    }

    #[test]
    fn call_macro() {
        let input = "%TEST { #00 } TEST";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(
            output,
            vec![
                ROMItem::MacroDef("TEST", vec![ROMItem::Byte(0x80), ROMItem::Byte(0x00)]),
                ROMItem::Macro("TEST")
            ]
        );
    }

    #[test]
    fn abs_pad_byte() {
        let input = "|70";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(output, vec![ROMItem::AbsPad(00, 0x70)]);
    }

    #[test]
    fn abs_pad_short() {
        let input = "|1970";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(output, vec![ROMItem::AbsPad(0x19, 0x70)]);
    }

    #[test]
    fn rel_pad() {
        let input = "$19";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(output, vec![ROMItem::RelPad(0x00, 0x19)]);
    }

    #[test]
    fn rel_pad_nibble() {
        let input = "$2";
        let state = State(HashMap::new());
        let stream = Stream { input, state };

        let output = parse_tal.parse(stream).unwrap();

        assert_eq!(output, vec![ROMItem::RelPad(0x00, 0x02)]);
    }
}
