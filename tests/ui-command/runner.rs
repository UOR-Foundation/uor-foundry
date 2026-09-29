use foundry_ui_command_core_probe::{commandBridgeBytes, effectWireBytes};
use std::{error::Error, fs};

fn decode_hex(text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if text.len() % 2 != 0 {
        return Err("odd hexadecimal length".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(Into::into))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() != 1 {
        return Err("one complete corpus required".into());
    }
    let mut count = 0;
    for line in fs::read_to_string(&arguments[0])?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err("closed vector format required".into());
        }
        let input = decode_hex(fields[2])?;
        let expected = decode_hex(fields[3])?;
        for _ in 0..2 {
            let actual = match fields[0] {
                "commandBridgeBytes" => commandBridgeBytes(input.clone()),
                "effectWireBytes" => effectWireBytes(input.clone()),
                _ => return Err("unknown generated root".into()),
            }
            .map_err(|error| format!("{}: {error:?}", fields[1]))?;
            assert_eq!(actual, expected, "{}", fields[1]);
        }
        count += 1;
        println!("PASS {}", fields[1]);
    }
    if count == 0 {
        return Err("empty corpus".into());
    }
    println!("PASS {count} complete vectors twice");
    Ok(())
}
