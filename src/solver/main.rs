use learn_pltl_fast::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// mod too_big_to_handle;

fn main() -> std::io::Result<()> {
    // let file = File::open("sample_(G(¬(x0)))∨(F((x0)∧(F(x1)))).ron")?;
    // let file = File::open("sample_0077.ron")?;
    // let file = File::open("sample197.ron")?;
    // let file = File::open("sample_tbth01.ron")?;
    // let file = File::open("sample_G((x0)∧((¬(x1))→((¬(x1))U((x2)∧(¬(x1)))))).ron")?;
    let file = File::open("sample_G((((x1)∧(¬(x2)))→(F(x6)))U((x2)∨(x5))).ron")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    if let Some(solution) = load_and_par_solve(contents) {
        // if let Some(solution) = par_mmztn_solve::<3>(&one_three_0077(), true) {
        println!("Solution: {}", solution);
    } else {
        println!("No solution found");
    }

    Ok(())
}

fn _load_and_solve(contents: Vec<u8>) -> Option<SyntaxTree> {
    // Ugly hack to get around limitations of deserialization for types with const generics.
    (1..=5).into_iter().find_map(|n| {
        match n {
            0 => ron::de::from_bytes::<Sample<0>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            1 => ron::de::from_bytes::<Sample<1>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            2 => ron::de::from_bytes::<Sample<2>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            3 => ron::de::from_bytes::<Sample<3>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            4 => ron::de::from_bytes::<Sample<4>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            5 => ron::de::from_bytes::<Sample<5>>(&contents)
                .map(|sample| brute_solve(&sample, true)),
            _ => panic!("out-of-bound parameter"),
        }
        .ok()
        .flatten()
    })
}

fn load_and_par_solve(contents: Vec<u8>) -> Option<SyntaxTree> {
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

// #[cfg(test)]
// mod satisfy {
//     use std::sync::Arc;

//     use super::*;

//     #[test]
//     fn sample_0077() {
//         let file = File::open("sample_0077.ron").expect("open file");
//         let mut buf_reader = BufReader::new(file);
//         let mut contents = Vec::new();
//         buf_reader.read_to_end(&mut contents).expect("read content");

//         let sample = ron::de::from_bytes::<Sample<3>>(&contents).expect("sample");
//         let solution = SyntaxTree::Binary {
//             op: BinaryOp::Or,
//             left_child: Arc::new(SyntaxTree::Unary {
//                 op: UnaryOp::Globally,
//                 child: Arc::new(SyntaxTree::Unary {
//                     op: UnaryOp::Not,
//                     child: Arc::new(SyntaxTree::Zeroary {
//                         op: ZeroaryOp::AtomicProp(0),
//                     }),
//                 }),
//             }),
//             right_child: Arc::new(SyntaxTree::Unary {
//                 op: UnaryOp::Finally,
//                 child: Arc::new(SyntaxTree::Binary {
//                     op: BinaryOp::And,
//                     left_child: Arc::new(SyntaxTree::Zeroary {
//                         op: ZeroaryOp::AtomicProp(0),
//                     }),
//                     right_child: Arc::new(SyntaxTree::Unary {
//                         op: UnaryOp::Finally,
//                         child: Arc::new(SyntaxTree::Zeroary {
//                             op: ZeroaryOp::AtomicProp(1),
//                         }),
//                     }),
//                 }),
//             }),
//         };
//         assert!(sample.is_consistent(&solution));
//     }

//     // #[test]
//     // fn tbth01() {
//     //     let file = File::open("sample_tbth01.ron").expect("open file");
//     //     let mut buf_reader = BufReader::new(file);
//     //     let mut contents = Vec::new();
//     //     buf_reader.read_to_end(&mut contents).expect("read content");

//     //     let de_sample = ron::de::from_bytes::<DeSample<3>>(&contents).expect("sample");
//     //     let sample = de_sample.into_sample();
//     //     let solution = SyntaxTree::Binary {
//     //         op: BinaryOp::Or,
//     //         left_child: Arc::new(
//     //             SyntaxTree::Unary {
//     //                 op: UnaryOp::Globally,
//     //                 child: Arc::new(
//     //                     SyntaxTree::Unary {
//     //                         op: UnaryOp::Not,
//     //                         child: Arc::new(
//     //                             SyntaxTree::Zeroary {
//     //                                 op: ZeroaryOp::AtomicProp(0)
//     //                             }
//     //                         )
//     //                     }
//     //                 )
//     //             }
//     //         ),
//     //         right_child: Arc::new(
//     //             SyntaxTree::Unary {
//     //                 op: UnaryOp::Finally,
//     //                 child: Arc::new(
//     //                     SyntaxTree::Binary {
//     //                         op: BinaryOp::And,
//     //                         left_child: Arc::new(
//     //                             SyntaxTree::Zeroary {
//     //                                 op: ZeroaryOp::AtomicProp(0)
//     //                             }
//     //                         ),
//     //                         right_child: Arc::new(
//     //                             SyntaxTree::Unary {
//     //                                 op: UnaryOp::Finally,
//     //                                 child: Arc::new(
//     //                                     SyntaxTree::Zeroary {
//     //                                         op: ZeroaryOp::AtomicProp(1)
//     //                                     }
//     //                                 )
//     //                             }
//     //                         )
//     //                     }
//     //                 )
//     //             }
//     //         )
//     //     };
//     //     assert!(sample.is_consistent(&solution));
//     // }
// }
