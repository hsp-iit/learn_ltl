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
        println!("Solution: {}", solution);
    } else {
        println!("No solution found");
    }

    Ok(())
}

fn load_and_solve(contents: Vec<u8>) -> Option<String> {
    // Ugly hack to get around limitations of deserialization for types with const generics.
    (1..).into_iter().find_map(|n| {
        match n {
            0 => ron::de::from_bytes::<Sample<0>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            1 => ron::de::from_bytes::<Sample<1>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            2 => ron::de::from_bytes::<Sample<2>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            3 => ron::de::from_bytes::<Sample<3>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            4 => ron::de::from_bytes::<Sample<4>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            5 => ron::de::from_bytes::<Sample<5>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            6 => ron::de::from_bytes::<Sample<6>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            7 => ron::de::from_bytes::<Sample<7>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            8 => ron::de::from_bytes::<Sample<8>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            9 => ron::de::from_bytes::<Sample<9>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            10 => ron::de::from_bytes::<Sample<10>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            11 => ron::de::from_bytes::<Sample<11>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            12 => ron::de::from_bytes::<Sample<12>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            13 => ron::de::from_bytes::<Sample<13>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            14 => ron::de::from_bytes::<Sample<14>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            15 => ron::de::from_bytes::<Sample<15>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            16 => ron::de::from_bytes::<Sample<16>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            17 => ron::de::from_bytes::<Sample<17>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            18 => ron::de::from_bytes::<Sample<18>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            19 => ron::de::from_bytes::<Sample<19>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            20 => ron::de::from_bytes::<Sample<20>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            21 => ron::de::from_bytes::<Sample<21>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            22 => ron::de::from_bytes::<Sample<22>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            23 => ron::de::from_bytes::<Sample<23>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            24 => ron::de::from_bytes::<Sample<24>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            25 => ron::de::from_bytes::<Sample<25>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            26 => ron::de::from_bytes::<Sample<26>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            27 => ron::de::from_bytes::<Sample<27>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            28 => ron::de::from_bytes::<Sample<28>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            29 => ron::de::from_bytes::<Sample<29>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            30 => ron::de::from_bytes::<Sample<30>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            31 => ron::de::from_bytes::<Sample<31>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            32 => ron::de::from_bytes::<Sample<32>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            33 => ron::de::from_bytes::<Sample<33>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            34 => ron::de::from_bytes::<Sample<34>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            35 => ron::de::from_bytes::<Sample<35>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            36 => ron::de::from_bytes::<Sample<36>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            37 => ron::de::from_bytes::<Sample<37>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            38 => ron::de::from_bytes::<Sample<38>>(&contents)
                .map(|sample| par_brute_solve(&sample, true).map(|formula| formula.print_w_named_vars(&sample.var_names))),
            _ => panic!("out-of-bound parameter"),
        }
        .ok()
        .flatten()
    })
}
