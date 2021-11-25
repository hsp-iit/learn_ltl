use crate::syntax::*;

pub type Trace<'a, const N: usize> = &'a [[bool; N]];

#[derive(Debug)]
pub struct Sample<'a, const N: usize> {
    pub positive_traces: Vec<Trace<'a, N>>,
    pub negative_traces: Vec<Trace<'a, N>>,
}

impl<'a, const N: usize> Sample<'a, N> {
    pub fn is_consistent(&self, formula: &SyntaxTree) -> bool {
        self.positive_traces.iter().all(|trace| formula.eval(trace))
            && self
                .negative_traces
                .iter()
                .all(|trace| !formula.eval(trace))
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

#[cfg(test)]
mod consistency {
    use super::*;

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
