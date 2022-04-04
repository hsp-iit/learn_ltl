use std::sync::Arc;

use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;

/// A tree structure with unary and binary nodes, but containing no data.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SkeletonTree {
    Leaf,
    UnaryNode(Box<SkeletonTree>),
    BinaryNode(Box<(SkeletonTree, SkeletonTree)>),
    ASNode(Vec<SkeletonTree>),
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
                    .map(|branch| SkeletonTree::UnaryNode(Box::new(branch)))
                    .collect();

                let size_tuples = SkeletonTree::gen_nums(size - 1, size);
                for size_tuple in size_tuples {
                    let skeleton_tuples = size_tuple
                        .into_iter()
                        .map(SkeletonTree::gen)
                        .multi_cartesian_product()
                        .map(|formulae| SkeletonTree::ASNode(formulae));
                    skeletons.extend(skeleton_tuples);
                }

                for left_size in 1..(size - 1) {
                    let left_smaller_skeletons = Self::gen(left_size);
                    let right_smaller_skeletons = Self::gen(size - 1 - left_size);

                    skeletons.extend(
                        left_smaller_skeletons
                            .into_iter()
                            .cartesian_product(right_smaller_skeletons.into_iter())
                            .map(|branches| SkeletonTree::BinaryNode(Box::new(branches))),
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
    fn gen_formulae<const N: usize>(&self) -> Vec<Arc<SyntaxTree>> {
        self.gen_formulae_partial::<N>(true, true, true, true)
    }

    fn gen_formulae_partial<const N: usize>(&self, tt: bool, ff: bool, and: bool, or: bool) -> Vec<Arc<SyntaxTree>> {
        match self {
            // Leaves of the `SkeletonTree` correspond to propositional variables
            SkeletonTree::Leaf if tt && ff => [Arc::new(SyntaxTree::False), Arc::new(SyntaxTree::True)].into_iter()
                .chain(
                    (0..N).map(|n| Arc::new(SyntaxTree::Atom(n as Idx)))
                ).collect::<Vec<Arc<SyntaxTree>>>(),
            SkeletonTree::Leaf if tt && !ff => [Arc::new(SyntaxTree::True)].into_iter()
                .chain(
                    (0..N).map(|n| Arc::new(SyntaxTree::Atom(n as Idx)))
                ).collect::<Vec<Arc<SyntaxTree>>>(),
            SkeletonTree::Leaf if !tt && ff => [Arc::new(SyntaxTree::False)].into_iter()
                .chain(
                    (0..N).map(|n| Arc::new(SyntaxTree::Atom(n as Idx)))
                ).collect::<Vec<Arc<SyntaxTree>>>(),
            SkeletonTree::Leaf => (0..N)
                .map(|n| Arc::new(SyntaxTree::Atom(n as Idx)))
                .collect::<Vec<Arc<SyntaxTree>>>(),
            // Unary nodes of the `SkeletonTree` correspond to unary operators of LTL
            SkeletonTree::UnaryNode(child) => {
                child.gen_formulae::<N>()
                    .into_iter()
                    .filter(|branch| check_next(branch.as_ref()))
                    .map(|child| Arc::new(SyntaxTree::Next(child)))
                    .collect()

                // let children = child.gen_formulae::<N>();
                // // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                // let mut trees = Vec::with_capacity(4 * children.len());

                // for child in children {
                //     let child = Box::new(child);

                //     // if check_not(child.as_ref()) {
                //     //     trees.push(SyntaxTree::Unary {
                //     //         op: UnaryOp::Not,
                //     //         child: child.clone(),
                //     //     });
                //     // }

                //     if check_next(child.as_ref()) {
                //         trees.push(SyntaxTree::Next(child));
                //     }

                //     // if check_globally(child.as_ref()) {
                //     //     trees.push(SyntaxTree::Unary {
                //     //         op: UnaryOp::Globally,
                //     //         child: child.clone(),
                //     //     });
                //     // }

                //     // if check_finally(child.as_ref()) {
                //     //     trees.push(SyntaxTree::Unary {
                //     //         op: UnaryOp::Finally,
                //     //         child,
                //     //     });
                //     // }
                // }

                // trees.shrink_to_fit();

                // trees
            }
            // Binary nodes of the `SkeletonTree` correspond to binary operators of LTL
            SkeletonTree::BinaryNode(child) => {
                child.0.gen_formulae::<N>()
                    .into_iter()
                    .cartesian_product(child.1.gen_formulae::<N>().into_iter())
                    .filter(|(left_branch, right_branch)| check_until(left_branch.as_ref(), right_branch.as_ref()))
                    .map(|children| Arc::new(SyntaxTree::Until(children.0, children.1)))
                    .collect()

                // let left_children = child.0.gen_formulae::<N>();
                // let right_children = child.1.gen_formulae::<N>();
                // // Use known bounds to allocate just as much memory as needed and avoid reallocations.
                // let mut trees = Vec::with_capacity(2 * left_children.len() * right_children.len());
                // let children = left_children
                //     .into_iter()
                //     .cartesian_product(right_children.into_iter());

                // for (left_child, right_child) in children {
                //     let children = Box::new((left_child, right_child));

                //     // if check_implies(children.as_ref()) {
                //     //     trees.push(SyntaxTree::Binary {
                //     //         op: BinaryOp::Implies,
                //     //         children: children.clone(),
                //     //     });
                //     // }

                //     if check_until(children.as_ref()) {
                //         trees.push(SyntaxTree::Until(children));
                //     }
                // }

                // trees.shrink_to_fit();

                // trees
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
                                    .gen_formulae_partial::<N>(false, false, false, false)
                                    .into_iter()
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                if check_and(&subformulae) {
                                    Some(Arc::new(SyntaxTree::And(subformulae)))
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
                                    .gen_formulae_partial::<N>(true, false, true, false)
                                    .into_iter()
                                    .combinations(multiplicity)
                            })
                            .multi_cartesian_product()
                            .filter_map(|tuples_of_subformulae| {
                                let subformulae = tuples_of_subformulae.concat();
                                // if check_xor(&subformulae) {
                                    Some(Arc::new(SyntaxTree::XOr(subformulae)))
                                // } else {
                                //     None
                                // }
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
                .map(|formula| formula.as_ref().clone())
        } else {
            SkeletonTree::gen(size)
                .into_par_iter()
                // .flat_map_iter(|skeleton| skeleton.gen_formulae::<N>())
                // .find_map_any(|skeleton| skeleton.gen_formulae::<N>().into_iter().find(|formula| sample.is_consistent(formula)))
                .flat_map(|skeleton| skeleton.gen_formulae::<N>())
                .find_any(|formula| sample.is_consistent(formula))
                .map(|formula| formula.as_ref().clone())
        }
    })
}


// Knuth-Bendix completion
// computed by https://smimram.github.io/ocaml-alg/kb/

// R1 : xor(x,ff()) -> x
// R2 : xor(x,x) -> ff()
// R3 : and(x,tt()) -> x
// R4 : and(x,ff()) -> ff()
// R5 : and(x,xor(y,z)) -> xor(and(x,y),and(x,z))
// R6 : next(tt()) -> tt()
// R7 : next(ff()) -> ff()
// R8 : next(xor(x,y)) -> xor(next(x),next(y))
// R9 : next(and(x,y)) -> and(next(x),next(y))
// R10 : next(until(x,y)) -> until(next(x),next(y))
// R11 : until(x,ff()) -> ff()
// R12 : and(until(x,y),until(z,y)) -> until(and(x,z),y)
// R13 : until(x,tt()) -> tt()

fn check_next(child: &SyntaxTree) -> bool {
    matches!(
        child,
        SyntaxTree::Atom(_)
    )
}

fn check_and_bin(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    match (left_child, right_child) {
        // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψ left_child: l_1
        (SyntaxTree::Until(_, c_1), SyntaxTree::Until(_, c_2)) if c_1 == c_2 => false,
        _ => true,
    }
}

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

fn check_until(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    // φ U φ ≡ φ
    left_child != right_child
        && match (left_child, right_child) {
            (_, SyntaxTree::True | SyntaxTree::False)
            // False U φ ≡ φ
            | (SyntaxTree::False, _) => false,
            // φ U ψ ≡ φ U (φ U ψ)
            (
                left_child,
                SyntaxTree::Until(branch, _),
            ) if left_child == branch.as_ref() => false,
            _ => true,
        }
}

// fn check_xor(branches: &[SyntaxTree]) -> bool {
//     branches.iter().all(|branch| !matches!(branch, SyntaxTree::False))
// }

fn check_and(branches: &[Arc<SyntaxTree>]) -> bool {
    branches.iter().tuple_combinations().all(|(left_branch, right_branch)| check_and_bin(left_branch, right_branch))
}

// TODO: write tests for checks
#[cfg(test)]
mod learn {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn gen_skeletons() {
        for sk in SkeletonTree::gen(5) {
            println!("{sk:?}");
        }
    }

    #[test]
    fn gen_formulae() {
        let sk = SkeletonTree::ASNode(vec![
            SkeletonTree::BinaryNode(Box::new((
                SkeletonTree::Leaf,
                SkeletonTree::Leaf,
            ))),
            SkeletonTree::UnaryNode(
                Box::new(SkeletonTree::Leaf)
            )
        ]);
        
        for formula in sk.gen_formulae::<2>() {
            println!("{formula}");
        }
    }

    #[test]
    pub fn num_gen_formulae() {
        const VARS: usize = 5;

        println!("size of formulae: {}B", size_of::<SyntaxTree>());

        for size in 1..12 {
            let formulae = SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<VARS>())
                .count();
            println!("Formulae of size {size}: {formulae} ({VARS} vars)");
        }
    }

    #[test]
    pub fn print_formulae() {
        const VARS: usize = 2;

        for size in 1..5 {
            for formula in SkeletonTree::gen(size)
                .into_iter()
                .flat_map(|skeleton| skeleton.gen_formulae::<VARS>())
            {
                println!("{formula}");
            }
        }
    }
}