use learn_pltl_fast::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod too_big_to_handle;

fn main() -> std::io::Result<()> {
    let file = File::open("sample_tbth01.ron")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    if let Some(solution) = load_and_solve(contents) {
        println!("Solution: {}", solution);
    } else {
        println!("No solution found");
    }

    Ok(())
}
