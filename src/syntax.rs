use serde::Deserialize;
use std::{fmt, sync::Arc};

pub type Time = u8;
pub type Idx = u8;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum UnaryOp {
    // Not,
    Next,
    // Globally,
    // Finally,
    // GloballyLeq(Time),
    // GloballyGneq(Time),
    // FinallyLeq(Time),
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            // UnaryOp::FinallyLeq(n) => write!(f, "F≤{}", n),
            // UnaryOp::Globally => write!(f, "G"),
            // UnaryOp::Finally => write!(f, "F"),
            // UnaryOp::GloballyGneq(n) => write!(f, "G>{}", n),
            // UnaryOp::GloballyLeq(n) => write!(f, "G≤{}", n),
            UnaryOp::Next => write!(f, "X"),
            // UnaryOp::Not => write!(f, "¬"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum BinaryOp {
    And,
    XOr,
    // Or,
    // Implies,
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
            // BinaryOp::Or => write!(f, "v"),
            BinaryOp::XOr => write!(f, "+"),
            // BinaryOp::Implies => write!(f, "→"),
            BinaryOp::Until => write!(f, "U"),
            // BinaryOp::Release => write!(f, "R"),
            // BinaryOp::ReleaseGneq(t) => write!(f, "R>{}", t),
            // BinaryOp::ReleaseLeq(t) => write!(f, "R≤{}", t),
            // BinaryOp::UntillLeq(t) => write!(f, "U≤{}", t),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum SyntaxTree {
    Atom(Idx),
    True,
    False,
    Unary {
        op: UnaryOp,
        child: Arc<SyntaxTree>,
    },
    Binary {
        op: BinaryOp,
        children: Arc<(SyntaxTree, SyntaxTree)>,
    },
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Atom(var) => write!(f, "x{}", var),
            SyntaxTree::True => write!(f, "T"),
            SyntaxTree::False => write!(f, "F"),
            SyntaxTree::Unary { op, child } => write!(f, "{}({})", op, child),
            SyntaxTree::Binary { op, children } => {
                write!(f, "({}){}({})", children.0, op, children.1)
            }
        }
    }
}

impl SyntaxTree {
    pub fn vars(&self) -> Idx {
        match self {
            SyntaxTree::Atom(n) => *n + 1,
            SyntaxTree::True| &SyntaxTree::False => 0,
            SyntaxTree::Unary { child, .. } => child.as_ref().vars(),
            SyntaxTree::Binary { children, .. } => children.0.vars().max(children.1.vars()),
        }
    }

    pub fn eval<const N: usize>(&self, trace: &[[bool; N]]) -> bool {
        match self {
            SyntaxTree::Atom(var) => trace
                .first()
                .map(|vals| {
                    vals.get(*var as usize)
                        .expect("interpret atomic proposition in trace")
                })
                .cloned()
                .unwrap_or(false),
                SyntaxTree::True => true,
                SyntaxTree::False => false,
                SyntaxTree::Unary { op, child } => match *op {
                // UnaryOp::Not => !child.eval(trace),
                UnaryOp::Next => {
                    if trace.is_empty() {
                        false
                    } else {
                        child.eval(&trace[1..])
                    }
                }
                // UnaryOp::Globally => (0..trace.len()).all(|t| child.eval(&trace[t..])),
                // UnaryOp::Finally => (0..trace.len()).any(|t| child.eval(&trace[t..])),
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
            SyntaxTree::Binary { op, children } => match *op {
                BinaryOp::And => children.0.eval(trace) && children.1.eval(trace),
                // BinaryOp::Or => children.0.eval(trace) || children.1.eval(trace),
                BinaryOp::XOr => children.0.eval(trace) != children.1.eval(trace) ,
                // BinaryOp::Implies => !children.0.eval(trace) || children.1.eval(trace),
                BinaryOp::Until => {
                    !trace.is_empty() && ( children.1.eval(trace) || ( children.0.eval(trace) && self.eval(&trace[1..]) ) )

                    // for t in 0..trace.len() {
                    //     let t_trace = &trace[t..];
                    //     if children.1.eval(t_trace) {
                    //         return true;
                    //     } else if !children.0.eval(t_trace) {
                    //         return false;
                    //     }
                    // }
                    // false
                } // BinaryOp::Release => {
                  //     // TODO: it's probably possible to optimize this
                  //     let release = (0..trace.len())
                  //         .find(|t| children.0.eval(&trace[*t..]))
                  //         .unwrap_or(trace.len());
                  //     (0..=release).all(|t| children.1.eval(&trace[t..]))
                  // }
                  // BinaryOp::ReleaseLeq(time) => {
                  //     let release = (0..=time as usize)
                  //         .find(|t| children.0.eval(&trace[(*t).min(trace.len())..]))
                  //         .unwrap_or(time as usize);
                  //     (0..=release).all(|t| children.1.eval(&trace[t.min(trace.len())..]))
                  // }
                  // BinaryOp::ReleaseGneq(time) => {
                  //     let release = (time as usize + 1..trace.len())
                  //         .find(|t| children.0.eval(&trace[(*t).min(trace.len())..]))
                  //         .unwrap_or(trace.len());
                  //     (time as usize + 1..=release)
                  //         .all(|t| children.1.eval(&trace[t.min(trace.len())..]))
                  // }
                  // BinaryOp::UntillLeq(time) => {
                  //     let until = (0..=time as usize)
                  //         .find(|t| children.1.eval(&trace[(*t).min(trace.len())..]))
                  //         .unwrap_or(time as usize + 1);
                  //     (0..until).all(|t| children.0.eval(&trace[t.min(trace.len())..]))
                  // }
            },
        }
    }
}

#[cfg(test)]
mod eval {
    use super::*;

    const ATOM_0: SyntaxTree = SyntaxTree::Atom(0);

    const ATOM_1: SyntaxTree = SyntaxTree::Atom(1);

    #[test]
    fn atomic_prop() {
        let trace = [[true]];
        assert!(ATOM_0.eval(&trace));

        let trace = [[false]];
        assert!(!ATOM_0.eval(&trace));
    }

    // #[test]
    // fn not() {
    //     let formula = SyntaxTree::Unary {
    //         op: UnaryOp::Not,
    //         child: Arc::new(ATOM_0),
    //     };

    //     let trace = [[false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true]];
    //     assert!(!formula.eval(&trace));
    // }

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

    // #[test]
    // fn globally() {
    //     let formula = SyntaxTree::Unary {
    //         op: UnaryOp::Globally,
    //         child: Arc::new(ATOM_0),
    //     };

    //     let trace = [[true], [true], [true]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true], [false], [true]];
    //     assert!(!formula.eval(&trace));
    // }

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
            children: Arc::new((ATOM_0, ATOM_1)),
        };

        let trace = [[true, true]];
        assert!(formula.eval(&trace));

        let trace = [[true, false]];
        assert!(!formula.eval(&trace));
    }

    // #[test]
    // fn or() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::Or,
    //         children: Arc::new((ATOM_0, ATOM_1)),
    //     };

    //     let trace = [[true, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[false, false]];
    //     assert!(!formula.eval(&trace));
    // }

    #[test]
    fn until() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::Until,
            children: Arc::new((ATOM_0, ATOM_1)),
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
    //         children.0: Arc::new(ATOM_0),
    //         children.1: Arc::new(ATOM_1),
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
    //         children.0: Arc::new(ATOM_0),
    //         children.1: Arc::new(ATOM_1),
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
    //         children.0: Arc::new(ATOM_0),
    //         children.1: Arc::new(ATOM_1),
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
    //         children.0: Arc::new(ATOM_0),
    //         children.1: Arc::new(ATOM_1),
    //     };

    //     let trace = [[true, false], [false, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false], [true, false], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false], [false, false]];
    //     assert!(!formula.eval(&trace));
    // }
}
