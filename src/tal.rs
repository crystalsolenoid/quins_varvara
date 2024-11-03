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
        .map(|token| {
            match token {
                "ADD" => "18",
                "SUB" => "19",
                "LIT" => "80",
                "DEO" => "17",
                "DEI" => "16",
                "BRK" => "00",
                other => other
            }
        })
        .collect::<String>()
}
