use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;

use std::sync::Arc;

/// A tree structure with unary and binary nodes, but containing no data.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SkeletonTree {
    Leaf,
    UnaryNode(Arc<SkeletonTree>),
    BinaryNode(Arc<(SkeletonTree, SkeletonTree)>),
    ASNode(Arc<Vec<SkeletonTree>>),
}

impl SkeletonTree {
    /// Generates all possible `SkeletonTree`s of the given size,
    /// where the size is given by the number of leaves.
    fn gen(size: usize) -> Vec<SkeletonTree> {
        match size {
            0 => panic!("No tree of size 0"),
            1 => vec![SkeletonTree::Leaf],
            size => {
                let smaller_skeletons = Self::gen(size - 1);
                let mut skeletons: Vec<SkeletonTree> = smaller_skeletons
                    .into_iter()
                    .map(|branch| SkeletonTree::UnaryNode(Arc::new(branch)))
                    .collect();

                let size_tuples = SkeletonTree::gen_nums(size - 1, size);
                for size_tuple in size_tuples {
                    let skeleton_tuples = size_tuple
                        .into_iter()
                        .map(SkeletonTree::gen)
                        .multi_cartesian_product()
                        .map(|formulae| SkeletonTree::ASNode(Arc::new(formulae)));
                    skeletons.extend(skeleton_tuples);
                }

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

    fn gen_nums(max_size: usize, target_size: usize) -> Vec<Vec<usize>> {
        let mut nums = Vec::new();
        for i in 1..=max_size.min(target_size) {
            if i == target_size {
                nums.push(vec![i]);
            } else if i < target_size - 1 {
                let mut sub_nums = SkeletonTree::gen_nums(i, target_size - i - 1);
                sub_nums.iter_mut().for_each(|nums| nums.push(i));
                nums.append(&mut sub_nums);
            }
        }
        nums.iter_mut().for_each(|tuple| tuple.sort());
        nums
    }

    /// Generates all possible LTL formulae whose structure fits that of the `SkeletonTree`,
    /// in the sense that leaves of the `SkeletonTree` correspond to propositional variables,
    /// unary nodes of the `SkeletonTree` correspond to unary operators of LTL,
    /// and binary nodes of the `SkeletonTree` correspond to binary operators of LTL.
    /// After being generated, a formula is checked under filtering criteria,
    /// and discarded if found to be equivalent to other formulae that have been or will included anyway.
    /// The const generic N represents the set of propositional variables which might appear in the generated formulae.
    fn gen_formulae<const N: usize>(&self) -> Vec<SyntaxTree> {
        self.gen_formulae_xxx::<N>(true, true)
    }

    fn gen_formulae_xxx<const N: usize>(&self, and: bool, or: bool) -> Vec<SyntaxTree> {
        match self {
            // Leaves of the `SkeletonTree` correspond to propositional variables
            SkeletonTree::Leaf => (0..N)
                .map(|n| SyntaxTree::Atom(n as Idx))
                .chain([SyntaxTree::Zeroary(false), SyntaxTree::Zeroary(true)])
                .collect::<Vec<SyntaxTree>>(),
            // Unary nodes of the `SkeletonTree` correspond to unary operators of LTL
            SkeletonTree::UnaryNode(child) => {
                let children = child.gen_formulae::<N>();
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(4 * children.len());

                for child in children {
                    let child = Arc::new(child);

                    // if check_not(child.as_ref()) {
                    //     trees.push(SyntaxTree::Unary {
                    //         op: UnaryOp::Not,
                    //         child: child.clone(),
                    //     });
                    // }

                    if check_next(child.as_ref()) {
                        trees.push(SyntaxTree::Next(child));
                    }

                    // if check_globally(child.as_ref()) {
                    //     trees.push(SyntaxTree::Unary {
                    //         op: UnaryOp::Globally,
                    //         child: child.clone(),
                    //     });
                    // }

                    // if check_finally(child.as_ref()) {
                    //     trees.push(SyntaxTree::Unary {
                    //         op: UnaryOp::Finally,
                    //         child,
                    //     });
                    // }
                }

                trees.shrink_to_fit();

                trees
            }
            // Binary nodes of the `SkeletonTree` correspond to binary operators of LTL
            SkeletonTree::BinaryNode(child) => {
                let left_children = child.0.gen_formulae::<N>();
                let right_children = child.1.gen_formulae::<N>();
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(2 * left_children.len() * right_children.len());
                let children = left_children
                    .into_iter()
                    .cartesian_product(right_children.into_iter());

                for (left_child, right_child) in children {
                    let children = Arc::new((left_child, right_child));

                    // if check_implies(children.as_ref()) {
                    //     trees.push(SyntaxTree::Binary {
                    //         op: BinaryOp::Implies,
                    //         children: children.clone(),
                    //     });
                    // }

                    if check_until(children.as_ref()) {
                        trees.push(SyntaxTree::Until(children));
                    }
                }

                trees.shrink_to_fit();

                trees
            }
            SkeletonTree::ASNode(leaves) => {
                use itertools::*;

                let mut formulae = Vec::new();

                if and {
                    formulae.extend(
                        leaves
                            .iter()
                            .dedup_with_count()
                            .map(|(multiplicity, skeleton)| {
                                skeleton
                                    .gen_formulae_xxx::<N>(false, false)
                                    .into_iter()
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                if check_and(&subformulae) {
                                    Some(SyntaxTree::And(Arc::new(subformulae)))
                                } else {
                                    None
                                }
                            }),
                    );
                }

                if or {
                    formulae.extend(
                        leaves
                            .iter()
                            .dedup_with_count()
                            .map(|(multiplicity, skeleton)| {
                                skeleton
                                    .gen_formulae_xxx::<N>(true, false)
                                    .into_iter()
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                if check_xor(&subformulae) {
                                    Some(SyntaxTree::XOr(Arc::new(subformulae)))
                                } else {
                                    None
                                }
                            }),
                    );
                }

                formulae
            }
        }
    }
}

// pub fn brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<SyntaxTree> {
//     (1..).into_iter().find_map(|size| {
//         if log {
//             println!("Searching formulae of size {}", size);
//         }
//         SkeletonTree::gen(size)
//             .into_iter()
//             .flat_map(|skeleton| skeleton.gen_formulae::<N>())
//             .find(|formula| sample.is_consistent(formula))
//     })
// }

// Parallel search is faster but less consistent then single-threaded search
pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<SyntaxTree> {
    use rayon::prelude::*;

    (1..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        // At small size, the overhead is not worth it.
        if size < 6 {
            SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<N>())
                .find(|formula| sample.is_consistent(formula))
        } else {
            SkeletonTree::gen(size)
                .into_par_iter()
                // .flat_map_iter(|skeleton| skeleton.gen_formulae::<N>())
                // .find_map_any(|skeleton| skeleton.gen_formulae::<N>().into_iter().find(|formula| sample.is_consistent(formula)))
                .flat_map(|skeleton| skeleton.gen_formulae::<N>())
                .find_any(|formula| sample.is_consistent(formula))
        }
    })
}

// fn check_not(child: &SyntaxTree) -> bool {
//     match child {
//         // ¬¬φ ≡ φ
//         SyntaxTree::Unary { op: UnaryOp::Not, .. }
//         // ¬(φ -> ψ) ≡ φ ∧ ¬ψ
//         | SyntaxTree::Binary { op: BinaryOp::Implies, .. } => false,
//         // ¬(¬φ ∨ ψ) ≡ φ ∧ ¬ψ
//         SyntaxTree::Binary { op: BinaryOp::Or, children }
//         // ¬(¬φ ∧ ψ) ≡ φ ∨ ¬ψ
//         | SyntaxTree::Binary { op: BinaryOp::And, children } if matches!(children.0, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
//         // ¬(φ ∨ ¬ψ) ≡ ¬φ ∧ ψ
//         SyntaxTree::Binary { op: BinaryOp::Or, children }
//         // ¬(φ ∧ ¬ψ) ≡ ¬φ ∨ ψ
//         | SyntaxTree::Binary { op: BinaryOp::And, children } if matches!(children.1, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
//         _ => true,
//     }
// }

fn check_next(child: &SyntaxTree) -> bool {
    !matches!(
        child,
        // X False ≡ False, X True ≡ True
        SyntaxTree::Zeroary(_)
        // X (x + y) -> Xx + Xy
        // X (x /\ y) -> Xx /\ Xy
        // X (x U y) -> Xx U Xy
        | SyntaxTree::Until(_)
        | SyntaxTree::And(_)
        | SyntaxTree::XOr(_)
    )
}

// fn check_globally(child: &SyntaxTree) -> bool {
//     !matches!(
//         child,
//         // G G φ <=> G φ
//         SyntaxTree::Unary { op: UnaryOp::Globally, .. }
//         // X G φ ≡ G X φ
//         | SyntaxTree::Unary { op: UnaryOp::Next, .. }
//         // ¬ F φ ≡ G ¬ φ
//         | SyntaxTree::Unary { op: UnaryOp::Finally, .. } // // G False ≡ False
//                                                          // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
//     )
// }

// fn check_finally(child: &SyntaxTree) -> bool {
//     !matches!(
//         child,
//         // F F φ <=> F φ
//         SyntaxTree::Unary { op: UnaryOp::Finally, .. }
//         // X F φ ≡ F X φ
//         | SyntaxTree::Unary { op: UnaryOp::Globally, .. } // // F False ≡ False
//                                                           // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
//     )
// }

// fn check_and((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
//     // Commutative law
//     left_child < right_child
//         && match (left_child, right_child) {
//         // 0 /\ x -> 0, 1 /\ x -> x
//         (SyntaxTree::Zeroary(_), _)
//         // x /\ 0 -> 0, x /\ 1 -> x
//         | (_, SyntaxTree::Zeroary(_))
//         // //  Excluded middle
//         // (child, SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child })
//         // |(SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child }, child) if child == neg_child.as_ref() => false,
//         // // // Domination law
//         // // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
//         // // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
//         // Associative laws
//         | (SyntaxTree::Binary { op: BinaryOp::And, .. }, ..) => false,
//         // // De Morgan's laws
//         // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
//         // // X (φ ∧ ψ) ≡ (X φ) ∧ (X ψ)
//         // | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. }) => false,
//         // // G (φ ∧ ψ)≡ (G φ) ∧ (G ψ)
//         // | (SyntaxTree::Unary { op: UnaryOp::Globally, .. }, SyntaxTree::Unary { op: UnaryOp::Globally, .. }) => false,
//         // // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
//         // // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
//         // (SyntaxTree::Binary { op: BinaryOp::Implies, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, children: c_2 }) if c_1.0 == c_2.0 || c_1.1 == c_2.1 => false,
//         // // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψ left_child: l_1
//         // (SyntaxTree::Binary { op: BinaryOp::Until, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Until, children: c_2, .. }) if c_1.1 == c_2.1 => false,
//         // // Absorption laws
//         // (SyntaxTree::Binary { op: BinaryOp::Or, children: c_1 }, right_child) if c_1.0 == *right_child || c_1.1 == *right_child => false,
//         // (left_child, SyntaxTree::Binary { op: BinaryOp::Or, children }) if children.0 == *left_child || children.1 == *left_child => false,
//         // // Distributive laws
//         // (SyntaxTree::Binary { op: BinaryOp::Or, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Or, children: c_2 }) if c_1.0 == c_2.0 || c_1.0 == c_2.1 || c_1.1 == c_2.0 || c_1.1 == c_2.1 => false,
//         // // G φ ≡ φ ∧ X(G φ)
//         // (
//         //     left_child,
//         //     SyntaxTree::Unary {
//         //         op: UnaryOp::Next,
//         //         child,
//         //     }
//         // ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
//         //     child.as_ref() != left_child
//         // } else {
//         //     true
//         // },
//         // // G φ ≡ X(G φ) ∧ φ
//         // (
//         //     SyntaxTree::Unary {
//         //         op: UnaryOp::Next,
//         //         child,
//         //     },
//         //     right_child,
//         // ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
//         //     child.as_ref() != right_child
//         // } else {
//         //     true
//         // },
//         _ => {
//             // x /\ (y + z) -> x + y /\ x + z
//             if let SyntaxTree::Binary { op: BinaryOp::XOr, .. } = left_child {
//                 return false;
//             }
//             // x /\ x -> x
//             // HARD: x could be any subformula of left- and right-hand-side!
//             let right_and_terms = list_and_terms(right_child);
//             !list_and_terms(left_child).iter().any(|and_term| right_and_terms.contains(and_term))
//         }
//     }
// }

// fn check_or((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
//     // Commutative law
//     left_child < right_child
//         && match (left_child, right_child) {
//         //  Excluded middle
//         (child, SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child })
//         |(SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child }, child) if child == neg_child.as_ref() => false,
//         // // Identity law
//         // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
//         // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
//         // Associative laws
//         | (SyntaxTree::Binary { op: BinaryOp::Or, .. }, ..)
//         // // De Morgan's laws
//         // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
//         // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
//         | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, ..)
//         // X (φ ∨ ψ) ≡ (X φ) ∨ (X ψ)
//         | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
//         // F (φ ∨ ψ) ≡ (F φ) ∨ (F ψ)
//         | (SyntaxTree::Unary { op: UnaryOp::Finally, .. }, SyntaxTree::Unary { op: UnaryOp::Finally, .. }) => false,
//         // (φ -> ψ_1) ∨ (φ -> ψ_2) ≡ φ -> (ψ_1 ∨ ψ_2)
//         // (φ_1 -> ψ) ∨ (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
//         (SyntaxTree::Binary { op: BinaryOp::Implies, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, children: c_2 }) if c_1.0 == c_2.0 || c_1.1 == c_2.1 => false,
//         // (φ U ψ_1) ∨ (φ U ψ_2) ≡ φ U (ψ_1 ∨ ψ_2)
//         (SyntaxTree::Binary { op: BinaryOp::Until, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Until, children: c_2 }) if c_1.0 == c_2.0 => false,
//         // Absorption laws
//         (SyntaxTree::Binary { op: BinaryOp::And, children }, right_child) if children.0 == *right_child || children.1 == *right_child => false,
//         (left_child, SyntaxTree::Binary { op: BinaryOp::And, children }) if children.0 == *left_child || children.1 == *left_child => false,
//         // Distributive laws
//         (SyntaxTree::Binary { op: BinaryOp::And, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::And, children: c_2 }) if c_1.0 == c_2.0 || c_1.0 == c_2.1 || c_1.1 == c_2.0 || c_1.1 == c_2.1 => false,
//         // F φ ≡ φ ∨ X(F φ)
//         (
//             left_child,
//             SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             }
//         ) => if let SyntaxTree::Unary { op: UnaryOp::Finally, child } = child.as_ref() {
//             child.as_ref() != left_child
//         } else {
//             true
//         },
//         // F φ ≡ X(F φ) ∨ φ
//         (
//             SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             },
//             right_child,
//         ) => if let SyntaxTree::Unary { op: UnaryOp::Finally, child } = child.as_ref() {
//             child.as_ref() != right_child
//         } else {
//             true
//         },
//         // φ U ψ ≡ ψ ∨ ( φ ∧ X(φ U ψ) )
//         // φ U ψ ≡ ψ ∨ ( X(φ U ψ) ∧ φ )
//         (
//             left_child,
//             SyntaxTree::Binary {
//                 op: BinaryOp::And,
//                 children: c_1,
//             }
//         ) => if let SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             } = &c_1.1 {
//                 if let SyntaxTree::Binary {
//                     op: BinaryOp::Until,
//                     children: c_2,
//                 } = child.as_ref() {
//                     !(*left_child == c_2.1 && c_1.0 == c_2.0)
//             } else if let SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             } = &c_1.0 {
//                 if let SyntaxTree::Binary {
//                     op: BinaryOp::Until,
//                     children: c_2,
//                 } = child.as_ref() {
//                     !(*left_child == c_2.1 && c_1.1 == c_2.0)
//                 } else {
//                     true
//                 }
//             } else {
//                 true
//             }
//         } else {
//             true
//         }
//         // φ U ψ ≡ ( φ ∧ X(φ U ψ) ) ∨ ψ
//         // φ U ψ ≡ ( X(φ U ψ) ∧ φ ) ∨ ψ
//         (
//             SyntaxTree::Binary {
//                 op: BinaryOp::And,
//                 children: c_1,
//             },
//             right_child
//         ) => if let SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             } = &c_1.1 {
//                 if let SyntaxTree::Binary {
//                     op: BinaryOp::Until,
//                     children: c_2,
//                 } = child.as_ref() {
//                     !(*right_child == c_2.1 && c_1.0 == c_2.0)
//             // // Made useless by commutativity optimization on ∧
//             } else if let SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             } = &c_1.0 {
//                 if let SyntaxTree::Binary {
//                     op: BinaryOp::Until,
//                     children: c_2,
//                 } = child.as_ref() {
//                     !(*right_child == c_2.1 && c_1.1 == c_2.0)
//                 } else {
//                     true
//                 }
//             } else {
//                 true
//             }
//         } else {
//             true
//         }
//         _ => true,
//     }
// }

// fn check_implies((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
//     !matches!(
//         (left_child, right_child),
//         // // Ex falso quodlibet (True defined as ¬False)
//         // (
//         //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
//         //     ..,
//         // )
//         // // φ -> False ≡ ¬φ
//         // | (
//         //     ..,
//         //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
//         // )
//         // // (SyntaxTree::Zeroary { op: ZeroaryOp::False, .. }, ..)
//         // // φ -> ψ ≡ ¬ψ -> ¬φ // subsumed by following rule
//         // (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
//         // ¬φ -> ψ ≡ ψ ∨ φ
//         | (
//             SyntaxTree::Unary {
//                 op: UnaryOp::Not,
//                 ..
//             },
//             ..,
//         )
//     )
// }

fn check_until((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
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
            // // X (φ U ψ) ≡ (X φ) U (X ψ)
            // (
            //     SyntaxTree::Unary {
            //         op: UnaryOp::Next, ..
            //     },
            //     SyntaxTree::Unary {
            //         op: UnaryOp::Next, ..
            //     },
            // ) => false,
            // φ U ψ ≡ φ U (φ U ψ)
            (
                left_child,
                SyntaxTree::Until(children),
            ) if *left_child == children.0 => false,
            _ => true,
        }
}

fn check_xor(branches: &[SyntaxTree]) -> bool {
    branches.iter().all(|branch| !matches!(branch, SyntaxTree::Zeroary(false)))
}

fn check_and(branches: &[SyntaxTree]) -> bool {
    branches.iter().all(|branch| !matches!(branch, SyntaxTree::Zeroary(_)))
}

// TODO: write tests for checks