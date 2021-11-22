use crate::SyntaxTree;

pub type Trace<'a, const N: usize> = &'a [[bool; N]];

pub struct Sample<'a, const N: usize> {
    positive_traces: Vec<Trace<'a, N>>,
    negative_traces: Vec<Trace<'a, N>>,
}

impl<'a, const N: usize> Sample<'a, N> {
    pub fn is_consistent(&self, formula: &SyntaxTree) -> bool {
        self.positive_traces.iter().all(|trace| formula.eval(trace))
            && self
                .negative_traces
                .iter()
                .all(|trace| !formula.eval(trace))
    }
}

#[cfg(test)]
mod consistency {
    use crate::*;

    const ATOM_0: SyntaxTree = SyntaxTree::Zeroary {
        op: ZeroaryOp::AtomicProp(0),
    };

    const ATOM_1: SyntaxTree = SyntaxTree::Zeroary {
        op: ZeroaryOp::AtomicProp(1),
    };

    #[test]
    fn and() {
        let sample = Sample {
            positive_traces: vec![&[[true, true]]],
            negative_traces: vec![&[[false, true]], &[[true, false]], &[[false, false]]],
        };

        let formula = SyntaxTree::Binary {
            op: BinaryOp::And,
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        assert!(sample.is_consistent(&formula));
    }
}
