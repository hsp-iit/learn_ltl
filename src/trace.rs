use crate::syntax::*;
use serde::{Deserialize, Serialize};
use serde_with::*;

pub type Trace<const N: usize> = Vec<[bool; N]>;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Sample<const N: usize> {
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub positive_traces: Vec<Trace<N>>,
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub negative_traces: Vec<Trace<N>>,
}

impl<const N: usize> Sample<N> {
    pub fn is_consistent(&self, formula: &SyntaxTree) -> bool {
        self.positive_traces
            .iter()
            .all(|trace| formula.eval(trace.as_slice()))
            && self
                .negative_traces
                .iter()
                .all(|trace| !formula.eval(trace.as_slice()))
    }

    pub fn time_lenght(&self) -> Time {
        let positive_lenght = self
            .positive_traces
            .iter()
            .map(|trace| trace.len())
            .max()
            .unwrap_or(0);
        let negative_lenght = self
            .negative_traces
            .iter()
            .map(|trace| trace.len())
            .max()
            .unwrap_or(0);
        positive_lenght.max(negative_lenght) as Time
    }
}

// #[cfg(test)]
// mod consistency {
//     use std::sync::Arc;

//     use super::*;

//     const ATOM_0: SyntaxTree = SyntaxTree::Zeroary {
//         op: ZeroaryOp::AtomicProp(0),
//     };

//     const ATOM_1: SyntaxTree = SyntaxTree::Zeroary {
//         op: ZeroaryOp::AtomicProp(1),
//     };

//     #[test]
//     fn and() {
//         let sample = Sample {
//             positive_traces: vec![vec![[true, true]]],
//             negative_traces: vec![
//                 vec![[false, true]],
//                 vec![[true, false]],
//                 vec![[false, false]],
//             ],
//         };

//         let formula = SyntaxTree::Binary {
//             op: BinaryOp::And,
//             left_child: Arc::new(ATOM_0),
//             right_child: Arc::new(ATOM_1),
//         };

//         assert!(sample.is_consistent(&formula));
//     }
// }
