use foundry_anonymous_ui_core_probe::{
    anonymousDesignBytes, anonymousNavigateBytes, anonymousNavigationSemanticBytes,
    anonymousPresentBytes, anonymousSelectorBytes, anonymousSemanticPresentBytes, present,
};
use std::{error::Error, fs};

fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() % 2 != 0
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("canonical lowercase hex required".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&value[at..at + 2], 16).map_err(Into::into))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("one complete corpus required".into());
    }
    let mut count = 0;
    for line in fs::read_to_string(&args[0])?.lines() {
        let row = line.split('\t').collect::<Vec<_>>();
        if row.len() != 4 || row[1].is_empty() {
            return Err("closed four-column corpus required".into());
        }
        let request = unhex(row[2])?;
        let expected = unhex(row[3])?;
        for _ in 0..2 {
            let result = match row[0] {
                "selector" => anonymousSelectorBytes(request.clone()),
                "presentation" => anonymousPresentBytes(request.clone()),
                "semantics" => anonymousSemanticPresentBytes(request.clone()),
                "navigation" => anonymousNavigateBytes(request.clone()),
                "navigation-presentation" => anonymousNavigationSemanticBytes(request.clone()),
                "design" => anonymousDesignBytes(request.clone()),
                "root" => present(request.clone()),
                _ => return Err("unknown generated entry".into()),
            }
            .map_err(|error| format!("{} {}: {error:?}", row[0], row[1]))?;
            assert_eq!(result, expected, "{} {}", row[0], row[1]);
        }
        println!("PASS {} {}", row[0], row[1]);
        count += 1;
    }
    if count == 0 {
        return Err("nonempty complete corpus required".into());
    }
    println!("PASS {count} complete vectors twice");
    Ok(())
}
