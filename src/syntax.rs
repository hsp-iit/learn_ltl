use crate::Trace;

pub type Time = u8;

// #[derive(Debug, Clone, Copy)]
// pub enum Operator {
//     Zeroary(ZeroaryOp),
//     Unary(UnaryOp),
//     Binary(BinaryOp),
// }

#[derive(Debug, Clone, Copy)]
pub enum ZeroaryOp {
    AtomicProp(usize),
    False,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Next,
    Globally,
    // GloballyLeq(Time),
    // GloballyGneq(Time),
    // FinallyLeq(Time),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    And,
    Or,
    // Release,
    // UntillLeq(Time),
    // ReleaseLeq(Time),
    // ReleaseGneq(Time),
}

pub enum SyntaxTree {
    Zeroary {
        op: ZeroaryOp,
    },
    Unary {
        op: UnaryOp,
        child: Box<SyntaxTree>,
    },
    Binary {
        op: BinaryOp,
        left_child: Box<SyntaxTree>,
        right_child: Box<SyntaxTree>,
    },
}

impl SyntaxTree {
    pub fn eval<'a, const N: usize>(&self, trace: Trace<'a, N>) -> bool {
        match self {
            SyntaxTree::Zeroary { op } => match op {
                ZeroaryOp::False => false,
                ZeroaryOp::AtomicProp(atomic_prop) => trace
                    .first()
                    .and_then(|vals| vals.get(*atomic_prop))
                    .cloned()
                    .unwrap_or(false),
            },
            SyntaxTree::Unary { op, child } => match op {
                UnaryOp::Not => !child.eval(trace),
                UnaryOp::Next => child.eval(&trace[1..]),
                UnaryOp::Globally => (0..trace.len()).all(|t| child.eval(&trace[t..])),
            },
            SyntaxTree::Binary {
                op,
                left_child,
                right_child,
            } => match op {
                BinaryOp::And => left_child.eval(trace) && right_child.eval(trace),
                BinaryOp::Or => left_child.eval(trace) || right_child.eval(trace),
            },
        }
    }
}

#[cfg(test)]
mod eval {
    use super::*;

    const FALSE: SyntaxTree = SyntaxTree::Zeroary {
        op: ZeroaryOp::False,
    };

    const ATOM_0: SyntaxTree = SyntaxTree::Zeroary {
        op: ZeroaryOp::AtomicProp(0),
    };

    const ATOM_1: SyntaxTree = SyntaxTree::Zeroary {
        op: ZeroaryOp::AtomicProp(1),
    };

    #[test]
    fn r#false() {
        let trace = [[]];
        assert!(!FALSE.eval(&trace));
    }

    #[test]
    fn atomic_prop() {
        let trace = [[true]];
        assert!(ATOM_0.eval(&trace));

        let trace = [[false]];
        assert!(!ATOM_0.eval(&trace));
    }

    #[test]
    fn not() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::Not,
            child: Box::new(ATOM_0),
        };

        let trace = [[false]];
        assert!(formula.eval(&trace));

        let trace = [[true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn next() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::Next,
            child: Box::new(ATOM_0),
        };

        let trace = [[false], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn globally() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::Globally,
            child: Box::new(ATOM_0),
        };

        let trace = [[true], [true], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn and() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::And,
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[true, true]];
        assert!(formula.eval(&trace));

        let trace = [[true, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn or() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::Or,
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[true, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, false]];
        assert!(!formula.eval(&trace));
    }
}
