use std::{fmt, sync::Arc};

// use std::num::NonZeroU8;
// pub type ZeroaryOp = Option<NonZeroU8>;

pub type Time = u8;
pub type Var = u8;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum ZeroaryOp {
    AtomicProp(Var),
    False,
}

impl fmt::Display for ZeroaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ZeroaryOp::AtomicProp(n) => write!(f, "x{}", n),
            ZeroaryOp::False => write!(f, "⊥"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Next,
    Globally,
    Finally,
    // GloballyLeq(Time),
    // GloballyGneq(Time),
    // FinallyLeq(Time),
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            // UnaryOp::FinallyLeq(n) => write!(f, "F≤{}", n),
            UnaryOp::Globally => write!(f, "G"),
            UnaryOp::Finally => write!(f, "F"),
            // UnaryOp::GloballyGneq(n) => write!(f, "G>{}", n),
            // UnaryOp::GloballyLeq(n) => write!(f, "G≤{}", n),
            UnaryOp::Next => write!(f, "X"),
            UnaryOp::Not => write!(f, "¬"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum BinaryOp {
    And,
    Or,
    Implies,
    Until,
    // Release,
    // ReleaseLeq(Time),
    // ReleaseGneq(Time),
    // UntillLeq(Time),
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BinaryOp::And => write!(f, "∧"),
            BinaryOp::Or => write!(f, "∨"),
            BinaryOp::Implies => write!(f, "→"),
            BinaryOp::Until => write!(f, "U"),
            // BinaryOp::Release => write!(f, "R"),
            // BinaryOp::ReleaseGneq(t) => write!(f, "R>{}", t),
            // BinaryOp::ReleaseLeq(t) => write!(f, "R≤{}", t),
            // BinaryOp::UntillLeq(t) => write!(f, "U≤{}", t),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum SyntaxTree {
    Zeroary {
        op: ZeroaryOp,
    },
    Unary {
        op: UnaryOp,
        child: Arc<SyntaxTree>,
    },
    Binary {
        op: BinaryOp,
        left_child: Arc<SyntaxTree>,
        right_child: Arc<SyntaxTree>,
    },
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Zeroary { op } => write!(f, "{}", op),
            SyntaxTree::Unary { op, child } => write!(f, "{}({})", op, child),
            SyntaxTree::Binary {
                op,
                left_child,
                right_child,
            } => write!(f, "({}){}({})", left_child, op, right_child),
        }
    }
}

impl SyntaxTree {
    pub fn eval<const N: usize>(&self, trace: &[[bool; N]]) -> bool {
        match self {
            SyntaxTree::Zeroary { op } => match op {
                ZeroaryOp::False => false,
                ZeroaryOp::AtomicProp(atomic_prop) => trace
                    .first()
                    .and_then(|vals| vals.get(*atomic_prop as usize))
                    .cloned()
                    .unwrap_or(false),
            },
            SyntaxTree::Unary { op, child } => match *op {
                UnaryOp::Not => !child.eval(trace),
                UnaryOp::Next => {
                    if trace.is_empty() {
                        false
                    } else {
                        child.eval(&trace[1..])
                    }
                }
                UnaryOp::Globally => (0..trace.len()).all(|t| child.eval(&trace[t..])),
                UnaryOp::Finally => (0..trace.len()).any(|t| child.eval(&trace[t..])),
                // UnaryOp::GloballyLeq(time) => {
                //     (0..(time as usize + 1).min(trace.len())).all(|t| child.eval(&trace[t..]))
                // }
                // UnaryOp::GloballyGneq(time) => {
                //     (time as usize + 1..trace.len()).all(|t| child.eval(&trace[t..]))
                // }
                // UnaryOp::FinallyLeq(time) => {
                //     (0..(time as usize + 1).min(trace.len())).any(|t| child.eval(&trace[t..]))
                // }
            },
            SyntaxTree::Binary {
                op,
                left_child,
                right_child,
            } => match *op {
                BinaryOp::And => left_child.eval(trace) && right_child.eval(trace),
                BinaryOp::Or => left_child.eval(trace) || right_child.eval(trace),
                BinaryOp::Implies => !left_child.eval(trace) || right_child.eval(trace),
                BinaryOp::Until => {
                    for t in 0..trace.len() {
                        let t_trace = &trace[t..];
                        if right_child.eval(t_trace) {
                            return true;
                        } else if !left_child.eval(t_trace) {
                            return false;
                        }
                    }
                    true
                }
                // BinaryOp::Release => {
                //     // TODO: it's probably possible to optimize this
                //     let release = (0..trace.len())
                //         .find(|t| left_child.eval(&trace[*t..]))
                //         .unwrap_or(trace.len());
                //     (0..=release).all(|t| right_child.eval(&trace[t..]))
                // }
                // BinaryOp::ReleaseLeq(time) => {
                //     let release = (0..=time as usize)
                //         .find(|t| left_child.eval(&trace[(*t).min(trace.len())..]))
                //         .unwrap_or(time as usize);
                //     (0..=release).all(|t| right_child.eval(&trace[t.min(trace.len())..]))
                // }
                // BinaryOp::ReleaseGneq(time) => {
                //     let release = (time as usize + 1..trace.len())
                //         .find(|t| left_child.eval(&trace[(*t).min(trace.len())..]))
                //         .unwrap_or(trace.len());
                //     (time as usize + 1..=release)
                //         .all(|t| right_child.eval(&trace[t.min(trace.len())..]))
                // }
                // BinaryOp::UntillLeq(time) => {
                //     let until = (0..=time as usize)
                //         .find(|t| right_child.eval(&trace[(*t).min(trace.len())..]))
                //         .unwrap_or(time as usize + 1);
                //     (0..until).all(|t| left_child.eval(&trace[t.min(trace.len())..]))
                // }
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
            child: Arc::new(ATOM_0),
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
            child: Arc::new(ATOM_0),
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
            child: Arc::new(ATOM_0),
        };

        let trace = [[true], [true], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));
    }

    // #[test]
    // fn globally_leq() {
    //     let formula = SyntaxTree::Unary {
    //         op: UnaryOp::GloballyLeq(1),
    //         child: Arc::new(ATOM_0),
    //     };

    //     let trace = [[true], [true], [false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true], [false], [true]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn globally_gneq() {
    //     let formula = SyntaxTree::Unary {
    //         op: UnaryOp::GloballyGneq(0),
    //         child: Arc::new(ATOM_0),
    //     };

    //     let trace = [[false], [true], [true]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true], [false], [true]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn finally_leq() {
    //     let formula = SyntaxTree::Unary {
    //         op: UnaryOp::FinallyLeq(1),
    //         child: Arc::new(ATOM_0),
    //     };

    //     let trace = [[false], [true], [false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[false], [false], [true]];
    //     assert!(!formula.eval(&trace));
    // }

    #[test]
    fn and() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::And,
            left_child: Arc::new(ATOM_0),
            right_child: Arc::new(ATOM_1),
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
            left_child: Arc::new(ATOM_0),
            right_child: Arc::new(ATOM_1),
        };

        let trace = [[true, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn until() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::Until,
            left_child: Arc::new(ATOM_0),
            right_child: Arc::new(ATOM_1),
        };

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(!formula.eval(&trace));
    }

    // #[test]
    // fn release() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::Release,
    //         left_child: Arc::new(ATOM_0),
    //         right_child: Arc::new(ATOM_1),
    //     };

    //     let trace = [[false, true], [true, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[false, true], [true, false], [false, true]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn release_leq() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::ReleaseLeq(1),
    //         left_child: Arc::new(ATOM_0),
    //         right_child: Arc::new(ATOM_1),
    //     };

    //     let trace = [[false, true], [false, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[false, true], [true, false], [false, false]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn release_gneq() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::ReleaseGneq(0),
    //         left_child: Arc::new(ATOM_0),
    //         right_child: Arc::new(ATOM_1),
    //     };

    //     let trace = [[false, false], [false, true], [true, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, true], [false, true], [true, false], [false, false]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn until_leq() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::UntillLeq(1),
    //         left_child: Arc::new(ATOM_0),
    //         right_child: Arc::new(ATOM_1),
    //     };

    //     let trace = [[true, false], [false, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false], [true, false], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false], [false, false]];
    //     assert!(!formula.eval(&trace));
    // }
}
