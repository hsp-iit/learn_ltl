use clap::Parser;
use learn_pltl_fast::*;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;

/// Generate a sample consistent with the given formula
#[derive(Parser, Debug)]
#[clap(name = "sampler")]
struct Sampler {
    /// Filename of the target formula
    #[arg(short, long)]
    formula: String,

    /// Number of positive traces
    #[arg(short, long)]
    positives: usize,

    /// Number of negative traces
    #[arg(short, long)]
    negatives: usize,

    /// Length of traces
    #[arg(short, long)]
    length: usize,
}

fn main() -> std::io::Result<()> {
    let sampler = Sampler::parse();

    let file = File::open(sampler.formula)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;
    let formula = ron::de::from_bytes::<SyntaxTree>(&contents).expect("formula");
    let vars = formula.vars();

    let name = format!("sample_{}.ron", formula);
    let file = File::create(name).expect("open sample file");
    let buf_writer = BufWriter::new(file);

    // Ugly hack to get around limitations of deserialization for types with const generics.
    match vars {
        0 => {
            let sample = sample::<0>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        1 => {
            let sample = sample::<1>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        2 => {
            let sample = sample::<2>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        3 => {
            let sample = sample::<3>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        4 => {
            let sample = sample::<4>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        5 => {
            let sample = sample::<5>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        6 => {
            let sample = sample::<6>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        7 => {
            let sample = sample::<7>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        8 => {
            let sample = sample::<8>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        9 => {
            let sample = sample::<9>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        10 => {
            let sample = sample::<10>(
                &formula,
                sampler.positives,
                sampler.negatives,
                sampler.length,
            );
            assert!(sample.is_consistent(&formula));
            ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
        }
        _ => panic!("out-of-bound parameter"),
    }

    Ok(())
}

fn sample<const N: usize>(
    formula: &SyntaxTree,
    positives: usize,
    negatives: usize,
    length: usize,
) -> Sample<N> {
    let mut sample = Sample::default();
    while sample.positive_traces() < positives || sample.negative_traces() < negatives {
        let trace = Vec::from_iter((0..length).map(|_| gen_bools()));
        let satisfaction = formula.eval(&trace);
        if satisfaction && sample.positive_traces() < positives {
            sample
                .add_positive_trace(trace)
                .expect("add positive trace");
        } else if !satisfaction && sample.negative_traces() < negatives {
            sample
                .add_negative_trace(trace)
                .expect("add negative trace");
        }
    }
    sample
}

fn gen_bools<const N: usize>() -> [bool; N] {
    use rand::prelude::*;
    let mut values = [true; N];
    rand::thread_rng().fill(&mut values[..]);
    values
}
