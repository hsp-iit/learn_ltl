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
            BinaryOp::Or => write!(f, "v"),
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
            SyntaxTree::Unary { op, child } => write!(f, "{}({})", op, child),
            SyntaxTree::Binary { op, children } => {
                write!(f, "({}){}({})", children.0, op, children.1)
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
                UnaryOp::Globally => (0..trace.len()).all(|t| child.eval(&trace[t..])),
                UnaryOp::Finally => (0..trace.len()).any(|t| child.eval(&trace[t..])),
            },
            SyntaxTree::Binary { op, children } => match *op {
                BinaryOp::And => children.0.eval(trace) && children.1.eval(trace),
                BinaryOp::Or => children.0.eval(trace) || children.1.eval(trace),
                BinaryOp::Implies => !children.0.eval(trace) || children.1.eval(trace),
                BinaryOp::Until => {
                    for t in 0..trace.len() {
                        let t_trace = &trace[t..];
                        if children.1.eval(t_trace) {
                            return true;
                        } else if !children.0.eval(t_trace) {
                            return false;
                        }
                    }
                    // Until is not satisfied if its right-hand-side argument never becomes true.
                    false
                }
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

    #[test]
    fn or() {
        let formula = SyntaxTree::Binary {
            op: BinaryOp::Or,
            children: Arc::new((ATOM_0, ATOM_1)),
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
            children: Arc::new((ATOM_0, ATOM_1)),
        };

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(!formula.eval(&trace));

        // Until is not satisfied if its right-hand-side argument never becomes true.
        let trace = [[true, false], [true, false], [true, false]];
        assert!(!formula.eval(&trace));
    }
}
