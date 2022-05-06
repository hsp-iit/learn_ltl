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
                .collect::<Vec<SyntaxTree>>(),
            // Unary nodes of the `SkeletonTree` correspond to unary operators of LTL
            SkeletonTree::UnaryNode(child) => {
                let children = child.gen_formulae::<N>();
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(4 * children.len());

                for child in children {
                    let child = Arc::new(child);

                    if check_not(child.as_ref()) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Not,
                            child: child.clone(),
                        });
                    }

                    if check_next(child.as_ref()) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Next,
                            child: child.clone(),
                        });
                    }

                    if check_globally(child.as_ref()) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Globally,
                            child: child.clone(),
                        });
                    }

                    if check_finally(child.as_ref()) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Finally,
                            child,
                        });
                    }
                }

                trees.shrink_to_fit();

                trees
            }
            // Binary nodes of the `SkeletonTree` correspond to binary operators of LTL
            SkeletonTree::BinaryNode(child) => {
                let left_children: Vec<Arc<SyntaxTree>> = child.0.gen_formulae::<N>().into_iter().map(|tree| Arc::new(tree)).collect();
                let right_children: Vec<Arc<SyntaxTree>> = child.1.gen_formulae::<N>().into_iter().map(|tree| Arc::new(tree)).collect();
                // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                let mut trees = Vec::with_capacity(2 * left_children.len() * right_children.len());
                let children = left_children
                    .into_iter()
                    .cartesian_product(right_children.into_iter());

                for (left_child, right_child) in children {
                    let left_ref = left_child.as_ref();
                    let right_ref = right_child.as_ref();

                    if check_implies(left_ref, right_ref) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::Implies,
                            children: (left_child.clone(), right_child.clone()),
                        });
                    }

                    if check_until(left_ref, right_ref) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::Until,
                            children: (left_child, right_child),
                        });
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
                                    .gen_formulae_xxx::<N>(false, true)
                                    .into_iter()
                                    .map(|tree| Arc::new(tree))
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                if check_multi_and(&subformulae) {
                                    Some(SyntaxTree::And(subformulae))
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
                                    .map(|tree| Arc::new(tree))
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                if check_multi_or(&subformulae) {
                                    Some(SyntaxTree::Or(subformulae))
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

/// Find a formula consistent with the given `Sample`.
/// Uses a fundamentally brute-force search algorithm.
// Parallel search is faster but less consistent then single-threaded search
pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<SyntaxTree> {
    use rayon::prelude::*;

    (1..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        // At small size, the overhead for parallel iterators is not worth it.
        // At larger size, we use parallel iterators for speed.
        if size < 6 {
            SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<N>())
                .find(|formula| sample.is_consistent(formula))
        } else {
            SkeletonTree::gen(size)
                .into_par_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<N>())
                .find_any(|formula| sample.is_consistent(formula))
        }
    })
}

// Priority: AND, OR, IMPLIES

fn check_not(child: &SyntaxTree) -> bool {
    match child {
        // ¬¬φ ≡ φ
        SyntaxTree::Unary { op: UnaryOp::Not, .. }
        // ¬(φ -> ψ) ≡ φ ∧ ¬ψ // BECAUSE OPERATOR PRIORITY
        | SyntaxTree::Binary { op: BinaryOp::Implies, .. }
        // ¬ F φ ≡ G ¬ φ
        | SyntaxTree::Unary { op: UnaryOp::Finally, .. } => false,
        // // ¬(¬φ ∨ ψ) ≡ φ ∧ ¬ψ
        // SyntaxTree::Binary { op: BinaryOp::Or, children } if matches!(children.0, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
        // ¬(¬φ ∧ ψ) ≡ φ ∨ ¬ψ
        // ¬(¬φ ∨ ψ) ≡ φ ∧ ¬ψ
        // | SyntaxTree::Binary { op: BinaryOp::And, children } if matches!(children.0, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
        SyntaxTree::And(branches)| SyntaxTree::Or(branches) => branches.iter().fold(0, |acc, formula| {
            if matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Not, .. }) {
                acc + 1
            } else {
                acc - 1
            }
        }) <= 0,
        _ => true,
    }
}

fn check_next(child: &SyntaxTree) -> bool {
    !matches!(
        child,
        // X ¬ φ ≡ ¬ X φ
        // X G φ ≡ G X φ
        // X F φ ≡ F X φ
        SyntaxTree::Unary {
            op: UnaryOp::Not | UnaryOp::Globally | UnaryOp::Finally,
            ..
        }
    )
}

fn check_globally(child: &SyntaxTree) -> bool {
    !matches!(
        child,
        // G G φ ≡ G φ
        SyntaxTree::Unary {
            op: UnaryOp::Globally,
            ..
        } // // X G φ ≡ G X φ
          // | SyntaxTree::Unary { op: UnaryOp::Next, .. }
          // // G False ≡ False
          // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
    )
}

fn check_finally(child: &SyntaxTree) -> bool {
    !matches!(
        child,
        // F F φ ≡ F φ
        SyntaxTree::Unary {
            op: UnaryOp::Finally,
            ..
        } // // X F φ ≡ F X φ
          // | SyntaxTree::Unary { op: UnaryOp::Next, .. }
          // // F False ≡ False
          // | SyntaxTree::Zeroary { op: ZeroaryOp::False }
    )
}

// fn check_and((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
//     // Commutative law WARNING: CORRECTNESS OF COMM+ASSOC IS NOT PROVEN
//     left_child < right_child
//     // left_child != right_child
//         && match (left_child, right_child) {
//         //  Excluded middle
//         (child, SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child })
//         |(SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child }, child) if child == neg_child.as_ref() => false,
//         // // Domination law
//         // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
//         // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
//         // Associative laws
//         | (SyntaxTree::Binary { op: BinaryOp::And, .. }, _)
//         // De Morgan's laws
//         | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
//         // X (φ ∧ ψ) ≡ (X φ) ∧ (X ψ)
//         | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
//         // G (φ ∧ ψ)≡ (G φ) ∧ (G ψ)
//         | (SyntaxTree::Unary { op: UnaryOp::Globally, .. }, SyntaxTree::Unary { op: UnaryOp::Globally, .. }) => false,
//         // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
//         // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
//         (SyntaxTree::Binary { op: BinaryOp::Implies, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, children: c_2 }) if c_1.0 == c_2.0 || c_1.1 == c_2.1 => false,
//         // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψ
//         (SyntaxTree::Binary { op: BinaryOp::Until, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Until, children: c_2, .. }) if c_1.1 == c_2.1 => false,
//         // Absorption laws
//         (SyntaxTree::Binary { op: BinaryOp::Or, children }, right_child) if children.0 == *right_child || children.1 == *right_child => false,
//         (left_child, SyntaxTree::Binary { op: BinaryOp::Or, children }) if children.0 == *left_child || children.1 == *left_child => false,
//         // Distributive laws
//         (SyntaxTree::Binary { op: BinaryOp::Or, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Or, children: c_2 }) if c_1.0 == c_2.0 || c_1.0 == c_2.1 || c_1.1 == c_2.0 || c_1.1 == c_2.1 => false,
//         // G φ ≡ φ ∧ X(G φ)
//         (
//             left_child,
//             SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             }
//         ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
//             child.as_ref() != left_child
//         } else {
//             true
//         },
//         // G φ ≡ X(G φ) ∧ φ
//         (
//             SyntaxTree::Unary {
//                 op: UnaryOp::Next,
//                 child,
//             },
//             right_child,
//         ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
//             child.as_ref() != right_child
//         } else {
//             true
//         },
//         _ => true,
//     }
// }

fn check_multi_and(branches: &Vec<Arc<SyntaxTree>>) -> bool {
    // ¬φ1 ∧ ¬φ2 ∧ ¬φ3 ∧ ψ1 ∧ ψ2 ∧ ψ3 ≡ ¬(φ1 ∨ φ2 ∨ φ3 ∨ ¬ψ1 ∨ ¬ψ2 ∨ ¬ψ3) ≡ ¬(ψ1 ∧ ψ2 ∧ ψ3 -> φ1 ∨ φ2 ∨ φ3)
    // (¬ φ) ∧ (¬ ψ) ≡ ¬ (φ ∨ ψ)
    branches.iter().filter(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Not, .. })).count() < 2
    // (X φ) ∧ (X ψ) ≡ X (φ ∧ ψ)
    && branches.iter().filter(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Next, .. })).count() < 2
    // (G φ) ∧ (G ψ) ≡ X (G ∧ ψ)
    && branches.iter().filter(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Globally, .. })).count() < 2
    && !branches.iter().permutations(2).any(|subformulae| {
        match (subformulae[0].as_ref(), subformulae[1].as_ref()) {
            (SyntaxTree::Or(children), right_child) => children.contains(subformulae[1]),
            // G φ ≡ φ ∧ X(G φ)
            (
                left_child,
                SyntaxTree::Unary {
                    op: UnaryOp::Next,
                    child,
                }
            ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
                child.as_ref() == left_child
            } else {
                false
            },
            _ => false,
        }
    })
    && !branches.iter().tuple_combinations().any(|(subformula_0, subformula_1)| {
        match (subformula_0.as_ref(), subformula_1.as_ref()) {
            // Distributive laws
            (SyntaxTree::Or(branches_l), SyntaxTree::Or(branches_r)) => branches_l.iter().any(|formula_l| branches_r.contains(formula_l) ),
            // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
            // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
            (SyntaxTree::Binary { op: BinaryOp::Implies, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, children: c_2 }) => c_1.0 == c_2.0 || c_1.1 == c_2.1,
            // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψ
            (SyntaxTree::Binary { op: BinaryOp::Until, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Until, children: c_2 }) => c_1.1 == c_2.1,
            // Absorption laws
            (SyntaxTree::Or(children), _) => children.contains(subformula_1),
            _ => false,
        }
    })
}

fn check_multi_or(branches: &Vec<Arc<SyntaxTree>>) -> bool {
    // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
    // ¬φ ∨ ¬φ ∨ ¬φ ∨ ψ ∨ ψ ∨ ψ ≡ φ ∧ φ ∧ φ -> ψ ∨ ψ ∨ ψ
    !branches.iter().any(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Not, .. }))
    // (X φ) ∨ (X ψ) ≡ X (φ ∨ ψ)
    && branches.iter().filter(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Next, .. })).count() < 2
    // (F φ) ∨ (F ψ) ≡ F (φ ∨ ψ)
    && branches.iter().filter(|formula| matches!(formula.as_ref(), &SyntaxTree::Unary { op: UnaryOp::Finally, .. })).count() < 2
    && !branches.iter().permutations(2).any(|subformulas| {
        let left_subformula = subformulas[0].as_ref();
        let right_subformula = subformulas[1].as_ref();

        match (left_subformula, right_subformula) {
            //  Excluded middle
            (child, &SyntaxTree::Unary { op: UnaryOp::Not, child: ref neg_child }) => child == neg_child.as_ref(),
            // Absorption laws, and
            // φ U ψ ≡ ψ ∨ ( φ ∧ X(φ U ψ) )
            // φ U ψ ≡ ψ ∨ ( X(φ U ψ) ∧ φ )
            (left_child, SyntaxTree::And(children)) => children.contains(subformulas[0]) || children.iter().permutations(2).any(|and_subformulas| {
                if let SyntaxTree::Unary {
                    op: UnaryOp::Next,
                    child,
                } = and_subformulas[1].as_ref() {
                    if let SyntaxTree::Binary {
                        op: BinaryOp::Until,
                        children: c_2,
                    } = child.as_ref() {
                        !(left_child == c_2.1.as_ref() && *and_subformulas[0] == c_2.0)
                    } else if let SyntaxTree::Unary {
                        op: UnaryOp::Next,
                        child,
                    } = and_subformulas[0].as_ref() {
                        if let SyntaxTree::Binary {
                            op: BinaryOp::Until,
                            children: c_2,
                        } = child.as_ref() {
                            left_child == c_2.1.as_ref() && *and_subformulas[1] == c_2.0
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }),
            // F φ ≡ φ ∨ X(F φ)
            (
                left_child,
                SyntaxTree::Unary {
                    op: UnaryOp::Next,
                    child,
                }
            ) => if let SyntaxTree::Unary { op: UnaryOp::Finally, child } = child.as_ref() {
                child.as_ref() == left_child
            } else {
                false
            },
            _ => false,
        }
    })
    && !branches.iter().tuple_combinations().any(|(subformula_0, subformula_1)| {
        match (subformula_0.as_ref(), subformula_1.as_ref()) {
            // Distributive laws
            (SyntaxTree::And(branches_l), SyntaxTree::And(branches_r)) => branches_l.iter().any(|formula_l| branches_r.contains(formula_l) ),
            // (φ -> ψ_1) ∨ (φ -> ψ_2) ≡ φ -> (ψ_1 ∨ ψ_2)
            // (φ_1 -> ψ) ∨ (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
            (SyntaxTree::Binary { op: BinaryOp::Implies, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, children: c_2 }) => c_1.0 == c_2.0 || c_1.1 == c_2.1,
            // (φ U ψ_1) ∨ (φ U ψ_2) ≡ φ U (ψ_1 ∨ ψ_2)
            (SyntaxTree::Binary { op: BinaryOp::Until, children: c_1 }, SyntaxTree::Binary { op: BinaryOp::Until, children: c_2 }) => c_1.0 == c_2.0,
            _ => false,
        }
    })
}

// fn check_or((left_child, right_child): &(SyntaxTree, SyntaxTree)) -> bool {
//     // Commutative law WARNING: CORRECTNESS OF COMM+ASSOC IS NOT PROVEN
//     left_child < right_child
//     // left_child != right_child
//         && match (left_child, right_child) {
//         //  Excluded middle
//         (child, SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child })
//         | (SyntaxTree::Unary { op: UnaryOp::Not, child: neg_child }, child) if child == neg_child.as_ref() => false,
//         // // Identity law
//         // (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
//         // | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
//         // Associative laws
//         | (SyntaxTree::Binary { op: BinaryOp::Or, .. }, _)
//         // // De Morgan's laws
//         // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
//         // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
//         | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, _)
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

fn check_implies(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    left_child != right_child
        && !matches!(
            (left_child, right_child),
            // // Ex falso quodlibet (True defined as ¬False)
            // (
            //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
            //     ..,
            // )
            // // φ -> False ≡ ¬φ
            // | (
            //     ..,
            //     SyntaxTree::Zeroary { op: ZeroaryOp::False },
            // )
            // // (SyntaxTree::Zeroary { op: ZeroaryOp::False, .. }, ..)
            // // φ -> ψ ≡ ¬ψ -> ¬φ // subsumed by following rule
            // (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
            // ¬φ -> ψ ≡ ψ ∨ φ
            (
            SyntaxTree::Unary {
                op: UnaryOp::Not,
                ..
            },
            _,
        )
        // φ -> ¬ψ ≡ ¬(ψ ∧ φ)
        | (
            _,
            SyntaxTree::Unary {
                op: UnaryOp::Not,
                ..
            }
        )
        // Currying
        // φ_1 -> (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
        | (
            _,
            SyntaxTree::Binary {
                op: BinaryOp::Implies,
                ..
            },
        )
        )
}

fn check_until(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
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
            (
                SyntaxTree::Unary {
                    op: UnaryOp::Next, ..
                },
                SyntaxTree::Unary {
                    op: UnaryOp::Next, ..
                },
            ) => false,
            // φ U ψ ≡ φ U (φ U ψ)
            (
                left_child,
                SyntaxTree::Binary {
                    op: BinaryOp::Until,
                    children,
                },
            ) if left_child == children.0.as_ref() => false,
            _ => true,
        }
}

// TODO: write tests for checks

#[cfg(test)]
mod learn {
    use super::*;

    #[test]
    fn formulae() {
        for size in 1..=9 {
            let formulae = SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<5>())
                .count();
            println!("formulae found (size {size}, vars 5): {formulae}");
        }
    }

    #[test]
    fn tuples() {
        assert_eq!(SkeletonTree::gen_nums(2, 3), vec![
            vec![1, 1],
        ]);
        assert_eq!(SkeletonTree::gen_nums(3, 4), vec![
            vec![1, 2],
        ]);
        assert_eq!(SkeletonTree::gen_nums(4, 5), vec![
            vec![1, 1, 1],
            vec![2, 2],
            vec![1, 3]
        ]);
    }

}
