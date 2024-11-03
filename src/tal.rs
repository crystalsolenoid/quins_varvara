use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use hex;

pub fn assemble(input: &str, output: &str) -> std::io::Result <()> {
    let input = File::open(input)?;
    let reader = BufReader::new(input);

    let hex: Vec<u8> = reader
        .lines()
        .filter_map(|e| e.ok())
        .map(|line| assemble_codes(&line))
        .map(|line| ascii_to_hex(&line))
        .flatten()
        .collect();

    std::fs::write(output, &hex)?;

    Ok(())
}

fn ascii_to_hex(input: &str) -> Vec<u8> {
    let mut ascii = input.as_bytes().to_owned();
    ascii.retain(|x| match x {
            b' ' | b'\n' => false,
            _ => true
        });

    let chunks = ascii.chunks_exact(2);
    if ascii.len() % 2 != 0 {
        panic!("Odd number of hex digits.");
    }

    chunks.map(|a| {
        let high = ascii_digit_to_u8(a[0]) << 4;
        let low = ascii_digit_to_u8(a[1]);
        high + low
    }).collect()
}

fn ascii_digit_to_u8(digit: u8) -> u8 {
    match digit {
        n if b'0' <= n && n <= b'9' => n - b'0',
        n if b'a' <= n && n <= b'f' => 10 + n - b'a',
        n if b'A' <= n && n <= b'F' => 10 + n - b'A',
        e => panic!("Unexpected character with ASCII code: {e}")
    }
}

fn assemble_codes(input: &str) -> String {
    input
        .split_ascii_whitespace()
        .map(|token| parse_code(token))
        .collect::<String>()
}

fn parse_code(token: &str) -> String {
    let token_bytes = token.as_bytes();

    if (token_bytes.len() < 3) | (token_bytes.len() > 6) {
        // assume it isn't an opcode and return unmodified
        return token.to_string()
    }

    let (opcode, flags) = token.split_at(3);

    let mut hex = match opcode {
        "ADD" => 0x18,
        "SUB" => 0x19,
        "LIT" => 0x80,
        "DEO" => 0x17,
        "DEI" => 0x16,
        "BRK" => 0x00,
        // assume it isn't an opcode and return unmodified
        _ => return token.to_string()
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

    return hex::encode(&[hex]);
}
