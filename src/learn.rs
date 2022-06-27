use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;

use std::sync::Arc;

/// A tree structure with unary and binary nodes, but containing no data.
#[derive(Debug, Clone)]
pub enum SkeletonTree {
    Leaf,
    UnaryNode(Arc<SkeletonTree>),
    BinaryNode(Arc<(SkeletonTree, SkeletonTree)>),
}

impl SkeletonTree {
    /// Generates all possible `SkeletonTree`s of the given size,
    /// where the size is given by the number of leaves.
    pub fn gen(size: usize) -> Vec<SkeletonTree> {
        match size {
            0 => panic!("No tree of size 0"),
            1 => vec![SkeletonTree::Leaf],
            size => {
                let smaller_skeletons = Self::gen(size - 1);
                let mut skeletons: Vec<SkeletonTree> = smaller_skeletons
                    .into_iter()
                    .map(|branch| SkeletonTree::UnaryNode(Arc::new(branch)))
                    .collect();
                for left_size in 1..(size - 1) {
                    let left_smaller_skeletons = Self::gen(left_size);
                    let right_smaller_skeletons = Self::gen(size - 1 - left_size);

                    skeletons.extend(
                        left_smaller_skeletons
                            .into_iter()
                            .cartesian_product(right_smaller_skeletons.into_iter())
                            .map(|branches| SkeletonTree::BinaryNode(Arc::new(branches))),
                    );
                }
                skeletons
            }
        }
    }

    /// Generates all possible LTL formulae whose structure fits that of the `SkeletonTree`,
    /// in the sense that leaves of the `SkeletonTree` correspond to propositional variables,
    /// unary nodes of the `SkeletonTree` correspond to unary operators of LTL,
    /// and binary nodes of the `SkeletonTree` correspond to binary operators of LTL.
    /// After being generated, a formula is checked under filtering criteria,
    /// and discarded if found to be equivalent to other formulae that have been or will included anyway.
    /// The const generic N represents the set of propositional variables which might appear in the generated formulae.
    pub fn gen_formulae<const N: usize>(&self, vars: &[Idx]) -> Vec<LTLformula> {
        match self {
            // Leaves of the `SkeletonTree` correspond to propositional variables
            SkeletonTree::Leaf => vars
                .into_iter()
                .map(|n| [LTLformula::Atom(true, *n), LTLformula::Atom(false, *n)])
                .flatten()
                .collect::<Vec<LTLformula>>(),
            // Unary nodes of the `SkeletonTree` correspond to unary operators of LTL
            SkeletonTree::UnaryNode(child) => {
                let children = child.gen_formulae::<N>(vars);
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(4 * children.len());

                for child in children {
                    let child = Arc::new(child);

                    // if check_not(child.as_ref()) {
                    //     trees.push(LTLformula::Not(child.clone()));
                    // }

                    if check_next(child.as_ref()) {
                        trees.push(LTLformula::Next(child.clone()));
                    }

                    if check_globally(child.as_ref()) {
                        trees.push(LTLformula::Globally(child));
                    }

                    // if check_finally(child.as_ref()) {
                    //     trees.push(LTLformula::Finally(child));
                    // }
                }

                trees.shrink_to_fit();

                trees
            }
            // Binary nodes of the `SkeletonTree` correspond to binary operators of LTL
            SkeletonTree::BinaryNode(child) => {
                let left_children: Vec<Arc<LTLformula>> = child
                    .0
                    .gen_formulae::<N>(vars)
                    .into_iter()
                    .map(Arc::new)
                    .collect();
                let right_children: Vec<Arc<LTLformula>> = child
                    .1
                    .gen_formulae::<N>(vars)
                    .into_iter()
                    .map(Arc::new)
                    .collect();
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(4 * left_children.len() * right_children.len());
                let children = left_children
                    .into_iter()
                    .cartesian_product(right_children.into_iter());

                for (left_child, right_child) in children {
                    if check_and(left_child.as_ref(), right_child.as_ref()) {
                        trees.push(LTLformula::And(left_child.clone(), right_child.clone()));
                    }

                    if check_or(left_child.as_ref(), right_child.as_ref()) {
                        trees.push(LTLformula::Or(left_child.clone(), right_child.clone()));
                    }

                    // if check_implies(left_child.as_ref(), right_child.as_ref()) {
                    //     trees.push(LTLformula::Implies(left_child.clone(), right_child.clone()));
                    // }

                    // if check_until(left_child.as_ref(), right_child.as_ref()) {
                    //     trees.push(LTLformula::Until(left_child.clone(), right_child.clone()));
                    // }

                    if check_release(left_child.as_ref(), right_child.as_ref()) {
                        trees.push(LTLformula::Release(left_child, right_child));
                    }
                }

                trees.shrink_to_fit();

                trees
            }
        }
    }
}

pub fn gen_formulae<const N: usize>(size: usize, vars: &[Idx]) -> Vec<LTLformula> {
    SkeletonTree::gen(size)
        .into_iter()
        .flat_map(|skeleton| skeleton.gen_formulae::<N>(vars))
        .collect_vec()
}

/// Find a formula consistent with the given `Sample`.
/// Uses a fundamentally brute-force search algorithm.
// Parallel search is faster but less consistent then single-threaded search
pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<LTLformula> {
    use rayon::prelude::*;

    if !sample.is_solvable() {
        return None;
    }

    let vars = &sample.vars();

    (1..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        // At small size, the overhead for parallel iterators is not worth it.
        // At larger size, we use parallel iterators for speed.
        if size < 6 {
            SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<N>(vars))
                .find(|formula| sample.is_consistent(formula))
        } else {
            SkeletonTree::gen(size)
                .into_par_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<N>(vars))
                .find_any(|formula| sample.is_consistent(formula))
        }
    })
}

// fn check_not(child: &LTLformula) -> bool {
//     match child {
//         // ¬¬φ ≡ φ
//         LTLformula::Not(_)
//         // // ¬(φ -> ψ) ≡ φ ∧ ¬ψ
//         // | LTLformula::Implies(_, _)
//         // ¬ F φ ≡ G ¬ φ
//         | LTLformula::Finally(_) => false,
//         // ¬(¬φ ∨ ψ) ≡ φ ∧ ¬ψ
//         LTLformula::Or(left_child, _)
//         // ¬(¬φ ∧ ψ) ≡ φ ∨ ¬ψ
//         | LTLformula::And(left_child, _) if matches!(left_child.as_ref(), LTLformula::Not(_)) => false,
//         // ¬(φ ∨ ¬ψ) ≡ ¬φ ∧ ψ
//         LTLformula::Or(_, right_child)
//         // ¬(φ ∧ ¬ψ) ≡ ¬φ ∨ ψ
//         | LTLformula::And(_, right_child) if matches!(right_child.as_ref(), LTLformula::Not(_)) => false,
//         _ => true,
//     }
// }

fn check_next(child: &LTLformula) -> bool {
    true
    // !matches!(
    //     child,
    //     // X ¬ φ ≡ ¬ X φ
    //     // X G φ ≡ G X φ
    //     // X F φ ≡ F X φ
    //     LTLformula::Not(_) | LTLformula::Globally(_) | LTLformula::Finally(_)
    // )
}

fn check_globally(child: &LTLformula) -> bool {
    !matches!(
        child,
        // G G φ ≡ G φ
        LTLformula::Globally(_)
        // X G φ ≡ G X φ
        | LTLformula::Next(_)
        // // G False ≡ False
        // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
    )
}

fn check_finally(child: &LTLformula) -> bool {
    !matches!(
        child,
        // F F φ ≡ F φ
        LTLformula::Finally(_)
        // X F φ ≡ F X φ
        | LTLformula::Next(_)
        // // F False ≡ False
        // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
    )
}

fn check_and(left_child: &LTLformula, right_child: &LTLformula) -> bool {
    // Commutative law WARNING: CORRECTNESS OF COMM+ASSOC IS NOT PROVEN
    left_child < right_child
    // left_child != right_child
        && match (left_child, right_child) {
        //  Excluded middle
        (child, LTLformula::Not(neg_child ))
        |(LTLformula::Not(neg_child), child) if child == neg_child.as_ref() => false,
        // // Domination law
        // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (LTLformula::And(_, _), _)
        // // De Morgan's laws
        // | (LTLformula::Not(_), LTLformula::Not(_))
        // X (φ ∧ ψ) ≡ (X φ) ∧ (X ψ)
        | (LTLformula::Next(_), LTLformula::Next(_))
        // G (φ ∧ ψ)≡ (G φ) ∧ (G ψ)
        | (LTLformula::Globally(_), LTLformula::Globally(_)) => false,
        // // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
        // // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
        // (LTLformula::Implies(c_1_0, c_1_1), LTLformula::Implies(c_2_0, c_2_1)) if c_1_0 == c_2_0 || c_1_1 == c_2_1 => false,
        // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψ
        (LTLformula::Until(_, c_1), LTLformula::Until(_, c_2)) if c_1 == c_2 => false,
        // (φ R ψ_1) ∧ (φ R ψ_2) ≡ φ R (ψ_1 ∧ ψ_2)
        (LTLformula::Release(c_1, _), LTLformula::Release(c_2, _)) if c_1 == c_2 => false,
        // Absorption laws
        (LTLformula::Or(c_0, c_1), right_child) if c_0.as_ref() == right_child || c_1.as_ref() == right_child => false,
        (left_child, LTLformula::Or(c_0, c_1)) if c_0.as_ref() == left_child || c_1.as_ref() == left_child => false,
        // Distributive laws
        (LTLformula::Or(c_1_0, c_1_1), LTLformula::Or(c_2_0, c_2_1)) if c_1_0 == c_2_0 || c_1_0 == c_2_1 || c_1_1 == c_2_0 || c_1_1 == c_2_1 => false,
        // G φ ≡ φ ∧ X(G φ)
        (
            left_child,
            LTLformula::Next(child)
        ) => if let LTLformula::Globally(child) = child.as_ref() {
            child.as_ref() != left_child
        } else {
            true
        },
        // G φ ≡ X(G φ) ∧ φ
        (
            LTLformula::Next(child),
            right_child,
        ) => if let LTLformula::Globally(child) = child.as_ref() {
            child.as_ref() != right_child
        } else {
            true
        },
        _ => true,
    }
}

fn check_or(left_child: &LTLformula, right_child: &LTLformula) -> bool {
    // Commutative law WARNING: CORRECTNESS OF COMM+ASSOC IS NOT PROVEN
    left_child < right_child
    // left_child != right_child
        && match (left_child, right_child) {
        //  Excluded middle
        (child, LTLformula::Not(neg_child))
        | (LTLformula::Not(neg_child), child) if child == neg_child.as_ref() => false,
        // // Identity law
        // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (LTLformula::Or(_, _), _)
        // // De Morgan's laws
        // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
        // // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
        // | (LTLformula::Not(_), _)
        // X (φ ∨ ψ) ≡ (X φ) ∨ (X ψ)
        | (LTLformula::Next(_), LTLformula::Next(_))
        // F (φ ∨ ψ) ≡ (F φ) ∨ (F ψ)
        | (LTLformula::Finally(_), LTLformula::Finally(_)) => false,
        // // (φ -> ψ_1) ∨ (φ -> ψ_2) ≡ φ -> (ψ_1 ∨ ψ_2)
        // // (φ_1 -> ψ) ∨ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
        // (LTLformula::Implies(c_1_0, c_1_1), LTLformula::Implies(c_2_0, c_2_1)) if c_1_0 == c_2_0 || c_1_1 == c_2_1 => false,
        // (φ U ψ_1) ∨ (φ U ψ_2) ≡ φ U (ψ_1 ∨ ψ_2)
        (LTLformula::Until(c_1, _), LTLformula::Until(c_2, _)) if c_1 == c_2 => false,
        // (φ_1 R ψ) ∨ (φ_2 R ψ) ≡ (φ_1 ∨ φ_2) R ψ
        (LTLformula::Release(_, c_1), LTLformula::Release(_, c_2)) if c_1 == c_2 => false,
        // Absorption laws
        (LTLformula::And(c_0, c_1), right_child) if c_0.as_ref() == right_child || c_1.as_ref() == right_child => false,
        (left_child, LTLformula::And(c_0, c_1)) if c_0.as_ref() == left_child || c_1.as_ref() == left_child => false,
        // Distributive laws
        (LTLformula::And(c_1_0, c_1_1), LTLformula::And(c_2_0, c_2_1)) if c_1_0 == c_2_0 || c_1_0 == c_2_1 || c_1_1 == c_2_0 || c_1_1 == c_2_1 => false,
        // F φ ≡ φ ∨ X(F φ)
        (
            left_child,
            LTLformula::Next(child)
        ) => if let LTLformula::Finally(child) = child.as_ref() {
            child.as_ref() != left_child
        } else {
            true
        },
        // F φ ≡ X(F φ) ∨ φ
        (
            LTLformula::Next(child),
            right_child,
        ) => if let LTLformula::Finally(child) = child.as_ref() {
            child.as_ref() != right_child
        } else {
            true
        },
        // φ U ψ ≡ ψ ∨ ( φ ∧ X(φ U ψ) )
        // φ U ψ ≡ ψ ∨ ( X(φ U ψ) ∧ φ )
        (
            left_child,
            LTLformula::And(c_1_0, c_1_1)
        ) => if let LTLformula::Next(child) = c_1_1.as_ref() {
                if let LTLformula::Until(c_2_0, c_2_1) | LTLformula::Release(c_2_0, c_2_1) = child.as_ref() {
                    !(left_child == c_2_1.as_ref() && c_1_0 == c_2_0)
            } else if let LTLformula::Next(child) = c_1_0.as_ref() {
                if let LTLformula::Until(c_2_0, c_2_1) | LTLformula::Release(c_2_0, c_2_1) = child.as_ref() {
                    !(left_child == c_2_1.as_ref() && c_1_1 == c_2_0)
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        }
        // φ U ψ ≡ ( φ ∧ X(φ U ψ) ) ∨ ψ
        // φ U ψ ≡ ( X(φ U ψ) ∧ φ ) ∨ ψ
        (
            LTLformula::And(c_1_0, c_1_1),
            right_child
        ) => if let LTLformula::Next(child) = c_1_1.as_ref() {
                if let LTLformula::Until(c_2_0, c_2_1) | LTLformula::Release(c_2_0, c_2_1) = child.as_ref() {
                    !(right_child == c_2_1.as_ref() && c_1_0 == c_2_0)
            } else if let LTLformula::Next(child) = c_1_0.as_ref() {
                if let LTLformula::Until(c_2_0, c_2_1) | LTLformula::Release(c_2_0, c_2_1) = child.as_ref() {
                    !(right_child == c_2_1.as_ref() && c_1_1 == c_2_0)
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        }
        _ => true,
    }
}

// fn check_implies(left_child: &LTLformula, right_child: &LTLformula) -> bool {
//     left_child != right_child
//         && !matches!(
//             (left_child, right_child),
//             // // Ex falso quodlibet (True defined as ¬False)
//             // (
//             //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
//             //     ..,
//             // )
//             // // φ -> False ≡ ¬φ
//             // | (
//             //     ..,
//             //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
//             // )
//             // // (SyntaxTree::Zeroary { op: ZeroaryOp::False, .. }, ..)
//             // // φ -> ψ ≡ ¬ψ -> ¬φ // subsumed by following rule
//             // (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
//             // ¬φ -> ψ ≡ ψ ∨ φ
//             (
//                 LTLformula::Not(_),
//                 _,
//             )
//             // φ -> ¬ψ ≡ ¬(ψ ∧ φ)
//             | (
//                 _,
//                 LTLformula::Not(_),
//             )
//             // Currying
//             // φ_1 -> (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
//             | (
//                 _,
//                 LTLformula::Implies(_, _),
//             )
//         )
// }

fn check_until(left_child: &LTLformula, right_child: &LTLformula) -> bool {
    // φ U φ ≡ φ
    left_child != right_child
        && match (left_child, right_child) {
            // // φ U False ≡ G φ
            // (
            //     ..,
            //     SyntaxTree::Zeroary {
            //         op: ZeroaryOp::False
            //     },
            // )
            // // False U φ ≡ φ
            // | (
            //     SyntaxTree::Zeroary {
            //         op: ZeroaryOp::False
            //     },
            //     ..
            // )
            // X (φ U ψ) ≡ (X φ) U (X ψ)
            (LTLformula::Next(_), LTLformula::Next(_)) => false,
            // φ U ψ ≡ φ U (φ U ψ)
            (left_child, LTLformula::Until(child, _)) if left_child == child.as_ref() => false,
            _ => true,
        }
}

fn check_release(left_child: &LTLformula, right_child: &LTLformula) -> bool {
    // φ R φ ≡ φ
    left_child != right_child
        && match (left_child, right_child) {
            // // φ U False ≡ G φ
            // (
            //     ..,
            //     SyntaxTree::Zeroary {
            //         op: ZeroaryOp::False
            //     },
            // )
            // // False U φ ≡ φ
            // | (
            //     SyntaxTree::Zeroary {
            //         op: ZeroaryOp::False
            //     },
            //     ..
            // )
            // X (φ R ψ) ≡ (X φ) R (X ψ)
            (LTLformula::Next(_), LTLformula::Next(_)) => false,
            // // φ R ψ ≡ φ R (φ R ψ)
            // (left_child, LTLformula::Until(child, _)) if left_child == child.as_ref() => false,
            _ => true,
        }
}
