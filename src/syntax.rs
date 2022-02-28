use serde::Deserialize;
use std::{fmt, sync::Arc};

/// The type representing time instants.
pub type Time = u8;

/// The type of indexes of propositional variables.
pub type Idx = u8;

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum UnaryOp {
    Not,
    Next,
    Globally,
    Finally,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            UnaryOp::Globally => write!(f, "G"),
            UnaryOp::Finally => write!(f, "F"),
            UnaryOp::Next => write!(f, "X"),
            UnaryOp::Not => write!(f, "¬"),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum BinaryOp {
    And,
    Or,
    Implies,
    Until,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BinaryOp::And => write!(f, "∧"),
            BinaryOp::Or => write!(f, "∨"),
            BinaryOp::Implies => write!(f, "→"),
            BinaryOp::Until => write!(f, "U"),
        }
    }
}

/// A formula represented via its syntax tree.
/// This is a recursive data structure, so it requires the use of smart pointers.
/// We use `Arc` to make it compatible with parallel computations.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum SyntaxTree {
    Atom(Idx),
    Not(Arc<SyntaxTree>),
    Next(Arc<SyntaxTree>),
    Globally(Arc<SyntaxTree>),
    Finally(Arc<SyntaxTree>),
    And(Arc<(SyntaxTree, SyntaxTree)>),
    Or(Arc<(SyntaxTree, SyntaxTree)>),
    Implies(Arc<(SyntaxTree, SyntaxTree)>),
    Until(Arc<(SyntaxTree, SyntaxTree)>),
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Atom(var) => write!(f, "x{}", var),
            SyntaxTree::Not(branch) => write!(f, "¬({branch})"),
            SyntaxTree::Next(branch) => write!(f, "X({branch})"),
            SyntaxTree::Globally(branch) => write!(f, "G({branch})"),
            SyntaxTree::Finally(branch) => write!(f, "F({branch})"),
            SyntaxTree::And(branches) => write!(f, "({})∧({})", branches.0, branches.1),
            SyntaxTree::Or(branches) => write!(f, "({})∨({})", branches.0, branches.1),
            SyntaxTree::Implies(branches) => write!(f, "({})→({})", branches.0, branches.1),
            SyntaxTree::Until(branches) => write!(f, "({})U({})", branches.0, branches.1),
        }
    }
}

impl SyntaxTree {
    /// Returns the highest propositional variable index appearing in the formula, plus 1.
    /// Used to count how many variables are needed to interpret the formula.
    pub fn vars(&self) -> Idx {
        match self {
            SyntaxTree::Atom(n) => *n + 1,
            SyntaxTree::Not(branch)
            | SyntaxTree::Next(branch)
            | SyntaxTree::Globally(branch)
            | SyntaxTree::Finally(branch) => branch.as_ref().vars(),
            SyntaxTree::And(branches)
            | SyntaxTree::Or(branches)
            | SyntaxTree::Implies(branches)
            | SyntaxTree::Until(branches) => branches.0.vars().max(branches.1.vars()),
        }
    }

    /// Evaluate a formula on a trace.
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
            SyntaxTree::Not(branch) => !branch.eval(trace),
            SyntaxTree::Next(branch) => {
                if trace.is_empty() {
                    false
                } else {
                    branch.eval(&trace[1..])
                }
            }
            SyntaxTree::Globally(branch) => (0..trace.len()).all(|t| branch.eval(&trace[t..])),
            SyntaxTree::Finally(branch) => (0..trace.len()).any(|t| branch.eval(&trace[t..])),
            SyntaxTree::And(branches) => branches.0.eval(trace) && branches.1.eval(trace),
            SyntaxTree::Or(branches) => branches.0.eval(trace) || branches.1.eval(trace),
            SyntaxTree::Implies(branches) => {
                let (left_branch, right_branch) = branches.as_ref();
                !left_branch.eval(trace) || right_branch.eval(trace)
            }
            SyntaxTree::Until(branches) => {
                let (left_branch, right_branch) = branches.as_ref();
                if trace.is_empty() {
                    false
                } else if right_branch.eval(trace) {
                    true
                } else if !left_branch.eval(trace) {
                    false
                } else {
                    self.eval(&trace[1..])
                }

                // Seems to be slightly slower, somehow?!?
                // for t in 0..trace.len() {
                //     let t_trace = &trace[t..];
                //     if children.1.eval(t_trace) {
                //         return true;
                //     } else if !children.0.eval(t_trace) {
                //         return false;
                //     }
                // }
                // // Until is not satisfied if its right-hand-side argument never becomes true.
                // false
            }
        }
    }
}

// #[cfg(test)]
// mod eval {
//     use super::*;

//     const ATOM_0: SyntaxTree = SyntaxTree::Atom(0);

//     const ATOM_1: SyntaxTree = SyntaxTree::Atom(1);

//     #[test]
//     fn atomic_prop() {
//         let trace = [[true]];
//         assert!(ATOM_0.eval(&trace));

//         let trace = [[false]];
//         assert!(!ATOM_0.eval(&trace));
//     }

//     #[test]
//     fn not() {
//         let formula = SyntaxTree::Unary {
//             op: UnaryOp::Not,
//             child: Arc::new(ATOM_0),
//         };

//         let trace = [[false]];
//         assert!(formula.eval(&trace));

//         let trace = [[true]];
//         assert!(!formula.eval(&trace));
//     }

//     #[test]
//     fn next() {
//         let formula = SyntaxTree::Unary {
//             op: UnaryOp::Next,
//             child: Arc::new(ATOM_0),
//         };

//         let trace = [[false], [true]];
//         assert!(formula.eval(&trace));

//         let trace = [[true], [false]];
//         assert!(!formula.eval(&trace));
//     }

//     #[test]
//     fn globally() {
//         let formula = SyntaxTree::Unary {
//             op: UnaryOp::Globally,
//             child: Arc::new(ATOM_0),
//         };

//         let trace = [[true], [true], [true]];
//         assert!(formula.eval(&trace));

//         let trace = [[true], [false], [true]];
//         assert!(!formula.eval(&trace));
//     }

//     #[test]
//     fn and() {
//         let formula = SyntaxTree::Binary {
//             op: BinaryOp::And,
//             children: Arc::new((ATOM_0, ATOM_1)),
//         };

//         let trace = [[true, true]];
//         assert!(formula.eval(&trace));

//         let trace = [[true, false]];
//         assert!(!formula.eval(&trace));
//     }

//     #[test]
//     fn or() {
//         let formula = SyntaxTree::Binary {
//             op: BinaryOp::Or,
//             children: Arc::new((ATOM_0, ATOM_1)),
//         };

//         let trace = [[true, false]];
//         assert!(formula.eval(&trace));

//         let trace = [[false, false]];
//         assert!(!formula.eval(&trace));
//     }

//     #[test]
//     fn until() {
//         let formula = SyntaxTree::Binary {
//             op: BinaryOp::Until,
//             children: Arc::new((ATOM_0, ATOM_1)),
//         };

//         let trace = [[true, false], [false, true], [false, false]];
//         assert!(formula.eval(&trace));

//         let trace = [[true, false], [true, false], [false, false]];
//         assert!(!formula.eval(&trace));

//         // Until is not satisfied if its right-hand-side argument never becomes true.
//         let trace = [[true, false], [true, false], [true, false]];
//         assert!(!formula.eval(&trace));
//     }
// }
