use learn_pltl_fast::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use clap::Parser;

/// Search for a formula consistent with the given sample.
#[derive(Parser, Debug)]
#[clap(name = "solver")]
struct Solver {
    /// Filename of the target sample.
    #[clap(short, long)]
    sample: String,
}

fn main() -> std::io::Result<()> {
    let solver = Solver::parse();

    let file = File::open(solver.sample)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    if let Some(solution) = load_and_solve(contents) {
        // if let Some(solution) = par_mmztn_solve::<3>(&one_three_0077(), true) {
        println!("Solution: {}", solution);
    } else {
        println!("No solution found");
    }

    Ok(())
}

// fn _load_and_solve(contents: Vec<u8>) -> Option<SyntaxTree> {
//     // Ugly hack to get around limitations of deserialization for types with const generics.
//     (1..=5).into_iter().find_map(|n| {
//         match n {
//             0 => {
//                 ron::de::from_bytes::<Sample<0>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             1 => {
//                 ron::de::from_bytes::<Sample<1>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             2 => {
//                 ron::de::from_bytes::<Sample<2>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             3 => {
//                 ron::de::from_bytes::<Sample<3>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             4 => {
//                 ron::de::from_bytes::<Sample<4>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             5 => {
//                 ron::de::from_bytes::<Sample<5>>(&contents).map(|sample| brute_solve(&sample, true))
//             }
//             _ => panic!("out-of-bound parameter"),
//         }
//         .ok()
//         .flatten()
//     })
// }

fn load_and_solve(contents: Vec<u8>) -> Option<SyntaxTree> {
    // Ugly hack to get around limitations of deserialization for types with const generics.
    (1..).into_iter().find_map(|n| {
        match n {
            0 => ron::de::from_bytes::<Sample<0>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            1 => ron::de::from_bytes::<Sample<1>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            2 => ron::de::from_bytes::<Sample<2>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            3 => ron::de::from_bytes::<Sample<3>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            4 => ron::de::from_bytes::<Sample<4>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            5 => ron::de::from_bytes::<Sample<5>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            6 => ron::de::from_bytes::<Sample<6>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            7 => ron::de::from_bytes::<Sample<7>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            8 => ron::de::from_bytes::<Sample<8>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            9 => ron::de::from_bytes::<Sample<9>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            10 => ron::de::from_bytes::<Sample<10>>(&contents)
                .map(|sample| par_brute_solve(&sample, true)),
            _ => panic!("out-of-bound parameter"),
        }
        .ok()
        .flatten()
    })
}
