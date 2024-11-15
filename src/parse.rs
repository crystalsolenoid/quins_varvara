use winnow::ascii::hex_digit1;
use winnow::error::{ContextError, ErrMode, ErrorKind, ParserError};
use winnow::stream::Stream;
use winnow::token::{any, one_of};
use winnow::{PResult, Parser};
use winnow::combinator::{repeat, alt};

use crate::opcode::{BASE_OPCODES, encode_base_code};

fn parse_base_opcode<'s>(input: &mut &'s str) -> PResult<&'s str> {
    alt(BASE_OPCODES).parse_next(input)
}

fn calculate_base_opcode(input: &mut & str) -> PResult<u8> {
    parse_base_opcode.map(|s: &str| encode_base_code(s)).parse_next(input)
}

fn calculate_flags(input: &mut &str) -> PResult<u8> {
    let flags = parse_opcode_flags.parse_next(input)?;
    let mut byte = 0;
    if flags.0 {
        byte = byte | 0b001_00000;
    }
    if flags.1 {
        byte = byte | 0b100_00000;
    }
    if flags.2 {
        byte = byte | 0b010_00000;
    }
    Ok(byte)
}

fn parse_opcode_flags<'s>(input: &mut &'s str) -> PResult<(bool, bool, bool)>{
    (parse_short_flag, parse_keep_flag, parse_return_flag).parse_next(input)

}

fn parse_short_flag<'s>(input: &mut &'s str) -> PResult<bool> {
    let flag = repeat(0..=1, "2").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_return_flag<'s>(input: &mut &'s str) -> PResult<bool>{
    let flag = repeat(0..=1, "r").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_keep_flag<'s>(input: &mut &'s str) -> PResult<bool>{
    let flag = repeat(0..=1, "k").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_opcode(input: &mut &str) -> PResult<u8> {
    let (base, flags) = (calculate_base_opcode, calculate_flags).parse_next(input)?;
    // TODO edge case error:
    // if base & flags != 0, that's when we return an error
    // because it means an invalid flag has been used
    // ie LITk
    Ok(base | flags)
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
fn parse_hexbyte(input: &mut &str) -> PResult<u8> {
    let high = one_of(HEX_DIGITS).parse_next(input)?;
    let low = one_of(HEX_DIGITS).parse_next(input)?;
    Ok((hex_digit_to_u8(high) << 4) + hex_digit_to_u8(low))
}

// TODO use an "in sequence" combinator?
fn parse_hexshort(input: &mut &str) -> PResult<u16> {
    let high: u16 = parse_hexbyte.parse_next(input)?.into();
    let low: u16 = parse_hexbyte.parse_next(input)?.into();
    let short = (high << 8) + low;
    Ok(short)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hexbyte() {
        let mut input = "fd .System/r";

        let output = parse_hexbyte.parse_next(&mut input).unwrap();

        assert_eq!(input, " .System/r");
        assert_eq!(output, 0xfd);

        assert!(parse_hexbyte.parse_next(&mut input).is_err());
    }

    #[test]
    fn hexshort() {
        let mut input = "4cfd .System/r";

        let output = parse_hexshort.parse_next(&mut input).unwrap();

        assert_eq!(input, " .System/r");
        assert_eq!(output, 0x4cfd);
    }

    #[test]
    fn single_digit_isnt_hexbyte() {
        let mut input = "d .System/r";
        let output = parse_hexbyte.parse_next(&mut input);
        assert!(output.is_err());
    }

    #[test]
    fn parses_opcode() {
        let mut input = "SUB2 ;on-frame";
        let output = parse_opcode.parse_next(&mut input).unwrap();
        assert_eq!(input, " ;on-frame");
        assert_eq!(output, 0x39);
    }

    #[test]
    fn parses_flags() {
        let mut input = "2k ;on-frame";
        let output = parse_opcode_flags.parse_next(&mut input).unwrap();
        assert_eq!(input, " ;on-frame");
        assert_eq!(output, (true,true,false));
    }

    #[test]
    fn parses_base_opcode_byte() {
        let mut input = "INC ;on-frame";
        let output = calculate_base_opcode.parse_next(&mut input).unwrap();
        assert_eq!(output, 0x01);
    }

    #[test]
    fn fails_on_opcode_then_nonsense() {
        let mut input = "SUB2abc ";
        let output = parse_opcode.parse_next(&mut input);
        assert!(output.is_err());
    }

    #[test]
    fn parses_base_opcode() {
        let mut input = "SUB2 ;on-frame";
        let output = parse_base_opcode.parse_next(&mut input).unwrap();
        assert_eq!(input, "2 ;on-frame");
        assert_eq!(output, "SUB");
    }

    #[test]
    fn errs_on_LITk() {
        let mut input = "LITk 1234";
        let output = parse_opcode.parse_next(&mut input);
        assert!(output.is_err());
    }
}
