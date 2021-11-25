use crate::trace::*;

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
    GloballyLeq(Time),
    GloballyGneq(Time),
    FinallyLeq(Time),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    And,
    Or,
    Release,
    ReleaseLeq(Time),
    ReleaseGneq(Time),
    UntillLeq(Time),
}

// Possible optimization: use Rc instead of Box
#[derive(Debug, Clone)]
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
    pub fn eval<const N: usize>(&self, trace: Trace<N>) -> bool {
        match self {
            SyntaxTree::Zeroary { op } => match op {
                ZeroaryOp::False => false,
                ZeroaryOp::AtomicProp(atomic_prop) => trace
                    .first()
                    .and_then(|vals| vals.get(*atomic_prop))
                    .cloned()
                    .unwrap_or(false),
            },
            SyntaxTree::Unary { op, child } => match *op {
                UnaryOp::Not => !child.eval(trace),
                UnaryOp::Next => {
                    if trace.len() > 0 {
                        child.eval(&trace[1..])
                    } else {
                        false
                    }
                }
                UnaryOp::Globally => (0..trace.len()).all(|t| child.eval(&trace[t..])),
                UnaryOp::GloballyLeq(time) => {
                    (0..(time as usize + 1).min(trace.len())).all(|t| child.eval(&trace[t..]))
                }
                UnaryOp::GloballyGneq(time) => {
                    (time as usize + 1..trace.len()).all(|t| child.eval(&trace[t..]))
                }
                UnaryOp::FinallyLeq(time) => {
                    (0..(time as usize + 1).min(trace.len())).any(|t| child.eval(&trace[t..]))
                }
            },
            SyntaxTree::Binary {
                op,
                left_child,
                right_child,
            } => match *op {
                BinaryOp::And => left_child.eval(trace) && right_child.eval(trace),
                BinaryOp::Or => left_child.eval(trace) || right_child.eval(trace),
                BinaryOp::Release => {
                    // TODO: it's probably possible to optimize this
                    let release = (0..trace.len())
                        .find(|t| left_child.eval(&trace[*t..]))
                        .unwrap_or(trace.len());
                    (0..=release).all(|t| right_child.eval(&trace[t..]))
                }
                BinaryOp::ReleaseLeq(time) => {
                    let release = (0..=time as usize)
                        .find(|t| left_child.eval(&trace[(*t).min(trace.len())..]))
                        .unwrap_or(time as usize);
                    (0..=release).all(|t| right_child.eval(&trace[t.min(trace.len())..]))
                }
                BinaryOp::ReleaseGneq(time) => {
                    let release = (time as usize + 1..trace.len())
                        .find(|t| left_child.eval(&trace[(*t).min(trace.len())..]))
                        .unwrap_or(trace.len());
                    (time as usize + 1..=release)
                        .all(|t| right_child.eval(&trace[t.min(trace.len())..]))
                }
                BinaryOp::UntillLeq(time) => {
                    let until = (0..=time as usize)
                        .find(|t| right_child.eval(&trace[(*t).min(trace.len())..]))
                        .unwrap_or(time as usize + 1);
                    (0..until).all(|t| left_child.eval(&trace[t.min(trace.len())..]))
                }
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
    fn globally_leq() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::GloballyLeq(1),
            child: Box::new(ATOM_0),
        };

        let trace = [[true], [true], [false]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn globally_gneq() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::GloballyGneq(0),
            child: Box::new(ATOM_0),
        };

        let trace = [[false], [true], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn finally_leq() {
        let formula = SyntaxTree::Unary {
            op: UnaryOp::FinallyLeq(1),
            child: Box::new(ATOM_0),
        };

        let trace = [[false], [true], [false]];
        assert!(formula.eval(&trace));

        let trace = [[false], [false], [true]];
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

    #[test]
    fn release() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::Release,
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[false, true], [true, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, true], [true, false], [false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn release_leq() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::ReleaseLeq(1),
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[false, true], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, true], [true, false], [false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn release_gneq() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::ReleaseGneq(0),
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[false, false], [false, true], [true, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, true], [false, true], [true, false], [false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn until_leq() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::UntillLeq(1),
            left_child: Box::new(ATOM_0),
            right_child: Box::new(ATOM_1),
        };

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [false, false]];
        assert!(!formula.eval(&trace));
    }
}
