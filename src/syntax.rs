use serde::Deserialize;
use std::fmt;

pub type Time = u8;
pub type Idx = u8;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum SyntaxTree {
    Atom(Idx),
    True,
    False,
    Next(Box<SyntaxTree>),
    Until(Box<(SyntaxTree, SyntaxTree)>),
    And(Vec<SyntaxTree>),
    XOr(Vec<SyntaxTree>),
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Atom(var) => write!(f, "x{}", var),
            SyntaxTree::True => write!(f, "T"),
            SyntaxTree::False => write!(f, "F"),
            SyntaxTree::Next(branch) => write!(f, "X({branch})"),
            SyntaxTree::Until(children) => {
                write!(f, "({})U({})", children.0, children.1)
            }
            SyntaxTree::And(branches) => {
                if let Some(branch) = branches.first() {
                    write!(f, "({branch})")?;
                    for branch in branches[1..].iter() {
                        write!(f, "*({branch})")?;
                    }
                }
                Ok(())
            }
            SyntaxTree::XOr(branches) => {
                if let Some(branch) = branches.first() {
                    write!(f, "({branch})")?;
                    for branch in branches[1..].iter() {
                        write!(f, "+({branch})")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl SyntaxTree {
    pub fn vars(&self) -> Idx {
        match self {
            SyntaxTree::Atom(n) => *n + 1,
            SyntaxTree::True | SyntaxTree::False => 0,
            SyntaxTree::Next(child) => child.as_ref().vars(),
            SyntaxTree::Until(children) => children.0.vars().max(children.1.vars()),
            SyntaxTree::And(branches) | SyntaxTree::XOr(branches) => branches.iter().map(SyntaxTree::vars).max().unwrap_or(0),
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
            SyntaxTree::Next(child) => !trace.is_empty() && child.eval(&trace[1..]),
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
            SyntaxTree::Until(children) => {
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
            }
            // BinaryOp::Release => {
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
            SyntaxTree::And(branches) => branches.iter().all(|branch| branch.eval(trace)),
            SyntaxTree::XOr(branches) => branches.iter().filter(|branch| branch.eval(trace)).count() == 1,
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
        let formula = SyntaxTree::Next(Box::new(ATOM_0));

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

    // #[test]
    // fn and() {
    //     let formula = SyntaxTree::Binary {
    //         op: BinaryOp::And,
    //         children: Arc::new((ATOM_0, ATOM_1)),
    //     };

    //     let trace = [[true, true]];
    //     assert!(formula.eval(&trace));

    //     let trace = [[true, false]];
    //     assert!(!formula.eval(&trace));
    // }

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
        let formula = SyntaxTree::Until(Box::new((ATOM_0, ATOM_1)));

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(!formula.eval(&trace));

        let trace = [[true, false], [true, false], [true, false]];
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
