use serde::Deserialize;
use std::{collections::BTreeSet, fmt, sync::Arc};

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
    Implies,
    Until,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BinaryOp::Implies => write!(f, "→"),
            BinaryOp::Until => write!(f, "U"),
        }
    }
}

pub type ASSet = BTreeSet<SyntaxTree>;

/// A formula represented via its syntax tree.
/// This is a recursive data structure, so it requires the use of smart pointers.
/// We use `Arc` to make it compatible with parallel computations.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum SyntaxTree {
    Atom(Idx),
    Unary {
        op: UnaryOp,
        child: Arc<SyntaxTree>,
    },
    Binary {
        op: BinaryOp,
        children: (Arc<SyntaxTree>, Arc<SyntaxTree>),
    },
    And(Vec<Arc<SyntaxTree>>),
    Or(Vec<Arc<SyntaxTree>>),
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Atom(var) => write!(f, "x{}", var),
            SyntaxTree::Unary { op, child } => write!(f, "{}({})", op, child),
            SyntaxTree::Binary { op, children } => {
                write!(f, "({}){}({})", children.0, op, children.1)
            }
            SyntaxTree::And(leaves) => {
                if leaves.is_empty() {
                    write!(f, "True")
                } else {
                    for (i, formula) in leaves.iter().enumerate() {
                        if i > 0 {
                            write!(f, "∧({formula})")?;
                        } else {
                            write!(f, "({formula})")?;
                        }
                    }
                    Ok(())
                }
            }
            SyntaxTree::Or(leaves) => {
                if leaves.is_empty() {
                    write!(f, "True")
                } else {
                    for (i, formula) in leaves.iter().enumerate() {
                        if i > 0 {
                            write!(f, "∨({formula})")?;
                        } else {
                            write!(f, "({formula})")?;
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

impl SyntaxTree {
    /// Returns the highest propositional variable index appearing in the formula, plus 1.
    /// Used to count how many variables are needed to interpret the formula.
    pub fn vars(&self) -> Idx {
        match self {
            SyntaxTree::Atom(n) => *n + 1,
            SyntaxTree::Unary { child, .. } => child.as_ref().vars(),
            SyntaxTree::Binary { children, .. } => children.0.vars().max(children.1.vars()),
            SyntaxTree::And(leaves) | SyntaxTree::Or(leaves) => {
                leaves.iter().map(|tree| tree.vars()).max().unwrap_or(0)
            }
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
            SyntaxTree::Unary { op, child } => match *op {
                UnaryOp::Not => !child.eval(trace),
                UnaryOp::Next => {
                    if trace.is_empty() {
                        false
                    } else {
                        child.eval(&trace[1..])
                    }
                }
                UnaryOp::Globally => (0..trace.len()).rev().all(|t| child.eval(&trace[t..])),
                UnaryOp::Finally => (0..trace.len()).rev().any(|t| child.eval(&trace[t..])),
            },
            SyntaxTree::Binary { op, children } => match *op {
                BinaryOp::Implies => !children.0.eval(trace) || children.1.eval(trace),
                BinaryOp::Until => {
                    if trace.is_empty() {
                        false
                    } else if children.1.eval(trace) {
                        true
                    } else if !children.0.eval(trace) {
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
            },
            SyntaxTree::And(leaves) => leaves.iter().all(|formula| formula.eval(trace)),
            SyntaxTree::Or(leaves) => leaves.iter().any(|formula| formula.eval(trace)),
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
    // fn and() {
    //     let formula = SyntaxTree::And(Arc::new(vec![ATOM_0, ATOM_1]));

    //     let trace = [[true, true]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn or() {
    //     let formula = SyntaxTree::Or(Arc::new(vec![ATOM_0, ATOM_1]));

    //     let trace = [[true, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[false, false]];
    //     assert!(!formula.eval(&trace));
    // }

    // #[test]
    // fn until() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::Until,
    //         children: Arc::new((ATOM_0, ATOM_1)),
    //     };

    //     let trace = [[true, false], [false, true], [false, false]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false], [true, false], [false, false]];
    //     assert!(!formula.eval(&trace));

    //     // Until is not satisfied if its right-hand-side argument never becomes true.
    //     let trace = [[true, false], [true, false], [true, false]];
    //     assert!(!formula.eval(&trace));
    // }
}
