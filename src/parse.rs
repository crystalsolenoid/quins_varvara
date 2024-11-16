use winnow::combinator::{alt, dispatch, fail, repeat, separated};
use winnow::stream::AsChar;
use winnow::token::{any, one_of, take_until, take_while};
use winnow::{PResult, Parser};

use crate::opcode::{encode_base_code, BASE_OPCODES};

pub fn parse_tal(input: &mut &str) -> PResult<Vec<u8>> {
    take_whitespace0.parse_next(input)?;
    let bytes: Vec<Vec<u8>> = separated(0.., next_tokens, take_whitespace1).parse_next(input)?;
    take_whitespace0.parse_next(input)?;
    Ok(bytes.into_iter().flatten().collect())
}

fn next_tokens(input: &mut &str) -> PResult<Vec<u8>> {
    let out =
        alt((parse_comment, parse_rune, parse_opcode, parse_many_hexbytes)).parse_next(input)?;
    Ok(out)
}

fn parse_todo(_input: &mut &str) -> PResult<Vec<u8>> {
    todo!();
}

fn parse_rune(input: &mut &str) -> PResult<Vec<u8>> {
    dispatch! {any;
        '%' => parse_todo,
        '|' => parse_todo,
        '$' => parse_todo,
        '@' => parse_todo,
        '&' => parse_todo,
        '#' => lit_rune,
        '.' => parse_todo,
        ',' => parse_todo,
        ';' => parse_todo,
        ':' => parse_todo,
        '\'' => parse_todo,
        '"' => parse_todo,
        _ => fail::<_, Vec<u8>, _>,
    }
    .parse_next(input)
}

fn lit_rune(input: &mut &str) -> PResult<Vec<u8>> {
    alt((lit_rune_short, lit_rune_byte)).parse_next(input)
}

fn lit_rune_byte(input: &mut &str) -> PResult<Vec<u8>> {
    let byte = parse_hexbyte.parse_next(input)?;
    Ok(vec![0x80, byte])
}

fn lit_rune_short(input: &mut &str) -> PResult<Vec<u8>> {
    let short = parse_hexshort.parse_next(input)?;
    Ok(vec![0xa0, short.0, short.1])
}

fn take_whitespace1<'s>(input: &mut &'s str) -> PResult<&'s str> {
    take_while(1.., (AsChar::is_space, AsChar::is_newline, '[', ']')).parse_next(input)
}

fn take_whitespace0<'s>(input: &mut &'s str) -> PResult<&'s str> {
    take_while(0.., (AsChar::is_space, AsChar::is_newline, '[', ']')).parse_next(input)
}

fn parse_comment(input: &mut &str) -> PResult<Vec<u8>> {
    ('(', take_until(0.., ')'), ')').parse_next(input)?;
    Ok(vec![])
}

fn parse_base_opcode<'s>(input: &mut &'s str) -> PResult<&'s str> {
    alt(BASE_OPCODES).parse_next(input)
}

fn calculate_base_opcode(input: &mut &str) -> PResult<u8> {
    parse_base_opcode
        .map(|s: &str| encode_base_code(s))
        .parse_next(input)
}

fn calculate_flags(input: &mut &str) -> PResult<u8> {
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

fn parse_opcode_flags(input: &mut &str) -> PResult<(bool, bool, bool)> {
    (parse_short_flag, parse_keep_flag, parse_return_flag).parse_next(input)
}

fn parse_short_flag(input: &mut &str) -> PResult<bool> {
    let flag = repeat(0..=1, "2").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_return_flag(input: &mut &str) -> PResult<bool> {
    let flag = repeat(0..=1, "r").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_keep_flag(input: &mut &str) -> PResult<bool> {
    let flag = repeat(0..=1, "k").map(|()| ()).take().parse_next(input)?;
    Ok(flag.len() == 1)
}

fn parse_opcode(input: &mut &str) -> PResult<Vec<u8>> {
    let (base, flags) = (calculate_base_opcode, calculate_flags).parse_next(input)?;
    // if base & flags != 0, that's when we return an error
    // because it means an invalid flag has been used
    // ie LITk
    if base & flags != 0 {
        return fail(input);
    }
    Ok(vec![base | flags])
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
    let byte = (hex_digit_to_u8(high) << 4) + hex_digit_to_u8(low);
    Ok(byte)
}

fn parse_many_hexbytes(input: &mut &str) -> PResult<Vec<u8>> {
    repeat(1.., parse_hexbyte).parse_next(input)
}

// TODO use an "in sequence" combinator?
fn parse_hexshort(input: &mut &str) -> PResult<(u8, u8)> {
    let high = parse_hexbyte.parse_next(input)?;
    let low = parse_hexbyte.parse_next(input)?;
    Ok((high, low))
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
        assert_eq!(output, (0x4c, 0xfd));
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
        assert_eq!(output, vec!(0x39));
    }

    #[test]
    fn parses_flags() {
        let mut input = "2k ;on-frame";
        let output = parse_opcode_flags.parse_next(&mut input).unwrap();
        assert_eq!(input, " ;on-frame");
        assert_eq!(output, (true, true, false));
    }

    #[test]
    fn parses_base_opcode_byte() {
        let mut input = "INC ;on-frame";
        let output = calculate_base_opcode.parse_next(&mut input).unwrap();
        assert_eq!(output, 0x01);
    }

    #[test]
    fn fails_on_opcode_then_nonsense() {
        let input = "SUB2abc ";
        let output = parse_tal.parse(input);
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

    #[test]
    fn comment() {
        let mut input = "( Comment ) SUB2 INC";
        let output = parse_comment.parse_next(&mut input).unwrap();
        assert_eq!(input, " SUB2 INC");
        assert_eq!(output, vec!());
    }

    #[test]
    fn parses_bytes_only_space_delimited() {
        let input = "a0 ff 80";
        let output = parse_tal.parse(input).unwrap();
        assert_eq!(output, vec!(0xa0, 0xff, 0x80));
    }

    #[test]
    fn parse_unseparated_bytes() {
        let input = "a0ff80";
        let output = parse_many_hexbytes.parse(input).unwrap();
        assert_eq!(output, vec!(0xa0, 0xff, 0x80));
    }

    #[test]
    fn parses_bytes() {
        let input = "a0 ff80";
        let output = parse_tal.parse(input).unwrap();
        assert_eq!(output, vec!(0xa0, 0xff, 0x80));
    }

    #[test]
    fn parses_bytes_and_opcodes() {
        let input = "a0 BRK ff80 LIT";
        let output = parse_tal.parse(input).unwrap();
        assert_eq!(output, vec!(0xa0, 0x00, 0xff, 0x80, 0x80));
    }

    #[test]
    fn ignores_comment() {
        let input = "a0 BRK (test comment!) ff80 LIT";
        let output = parse_tal.parse(input).unwrap();
        assert_eq!(output, vec!(0xa0, 0x00, 0xff, 0x80, 0x80));
    }

    #[test]
    fn errs_when_opcode_has_no_whitespace_before() {
        let input = "a0BRK ff80 LIT";
        let output = parse_tal.parse(input);
        assert!(output.is_err());
    }

    #[test]
    fn errs_when_opcode_has_no_whitespace_after() {
        let input = "a0 BRKff80 LIT";
        let output = parse_tal.parse(input);
        assert!(output.is_err());
    }

    #[test]
    fn lit_rune() {
        let input = "#10";
        let output = parse_tal.parse(input).unwrap();
        assert_eq!(output, vec!(0x80, 0x10));
    }

    #[test]
    fn lit_rune_short() {
        let output_rune = parse_tal.parse("#dcf2").unwrap();
        let output_opcode = parse_tal.parse("LIT2 dcf2").unwrap();
        assert_eq!(output_rune, output_opcode);
    }

    #[test]
    fn parses_multiline() {
        let input = "#2ce9 #08 DEO2\n";
        let output = parse_tal.parse(input).unwrap();
    }
}
