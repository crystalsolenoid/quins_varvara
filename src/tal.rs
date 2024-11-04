use hex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub fn assemble(input: &str, output: &str) -> std::io::Result<()> {
    let input = File::open(input)?;
    let reader = BufReader::new(input);

    let hex: Vec<u8> = reader
        .lines()
        .map_while(Result::ok)
        .map(assemble_codes)
        .flat_map(ascii_to_hex)
        .collect();

    std::fs::write(output, &hex)?;

    Ok(())
}

fn ascii_to_hex(input: String) -> Vec<u8> {
    let mut ascii = input.as_bytes().to_owned();
    ascii.retain(|x| !matches!(x, b' ' | b'\n'));

    let chunks = ascii.chunks_exact(2);
    if ascii.len() % 2 != 0 {
        panic!("Odd number of hex digits. {ascii:?}");
    }

    chunks
        .map(|a| {
            let high = ascii_digit_to_u8(a[0]) << 4;
            let low = ascii_digit_to_u8(a[1]);
            high + low
        })
        .collect()
}

fn ascii_digit_to_u8(digit: u8) -> u8 {
    match digit {
        n if n.is_ascii_digit() => n - b'0',
        n if (b'a'..=b'f').contains(&n) => 10 + n - b'a',
        n if (b'A'..=b'F').contains(&n) => 10 + n - b'A',
        e => panic!("Unexpected character with ASCII code: {e}"),
    }
}

fn assemble_codes(input: String) -> String {
    input
        .split_ascii_whitespace()
        .map(parse_code)
        .collect::<String>()
}

fn parse_code(token: &str) -> String {
    let token_bytes = token.as_bytes();

    if (token_bytes.len() < 3) | (token_bytes.len() > 6) {
        // assume it isn't an opcode and return unmodified
        return token.to_string();
    }

    let (opcode, flags) = token.split_at(3);

    let mut hex = match opcode {
        "ADD" => 0x18,
        "SUB" => 0x19,
        "LIT" => 0x80,
        "DEO" => 0x17,
        "DEI" => 0x16,
        "BRK" => 0x00,
        "INC" => 0x01,
        "MUL" => 0x1a,
        "DIV" => 0x1b,
        "SFT" => 0x1f,
        // assume it isn't an opcode and return unmodified
        _ => return token.to_string(),
    };

    // TODO don't let exceptions to the flag rules (like
    // BRK and LIT) permit the hex value to get above 0xFF.
    // Assert the order of the flags (2kr) for all present?
    if flags.contains("k") {
        hex += 0b100_00000;
    }

    if flags.contains("r") {
        hex += 0b010_00000;
    }

    if flags.contains("2") {
        hex += 0b001_00000;
    }

    hex::encode([hex])
}
