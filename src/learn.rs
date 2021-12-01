// use z3::ast::Bool;

use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;
use std::rc::Rc;
use std::sync::Arc;

// pub fn learn<const N: usize>(sample: Sample<N>) -> Option<SyntaxTree> {
//     unimplemented!();
// }

// pub fn learn_size<const N: usize>(sample: Sample<N>, size: usize) -> Option<SyntaxTree> {
//     unimplemented!();
// }

#[derive(Debug, Clone)]
enum SkeletonTree {
    Zeroary,
    Unary(Arc<SkeletonTree>),
    Binary((Arc<SkeletonTree>, Arc<SkeletonTree>)),
}

impl SkeletonTree {
    fn gen_formulae<const N: usize>(&self) -> Vec<SyntaxTree> {
        match self {
            SkeletonTree::Zeroary => {
                let mut trees = (0..N)
                    .map(|n| {
                        SyntaxTree::Zeroary {
                            op: ZeroaryOp::AtomicProp(n as Var),
                        }
                    })
                    .collect::<Vec<SyntaxTree>>();
                trees.push(SyntaxTree::Zeroary {
                    op: ZeroaryOp::False,
                });
                trees
            }
            SkeletonTree::Unary(child) => {
                let mut trees = Vec::new();
                let children = child.gen_formulae::<N>();
    
                for child in children {
                    let a_child = Arc::new(child.clone());
    
                    if check_globally(&child) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Globally,
                            child: a_child.clone(),
                        });
                    }
                    if check_finally(&child) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Finally,
                            child: a_child.clone(),
                        });
                    }
    
                    if check_not(&child) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Not,
                            child: a_child.clone(),
                        });
                    }
    
                    if check_next(&child) {
                        trees.push(SyntaxTree::Unary {
                            op: UnaryOp::Next,
                            child: a_child,
                        });
                    }
                }
    
                trees
            }
            SkeletonTree::Binary(child) => {
                let mut trees = Vec::new();
                let left_children = child.0.gen_formulae::<N>();
                let right_children = child.1.gen_formulae::<N>();
                let children = left_children
                    .into_iter()
                    .cartesian_product(right_children.into_iter());
    
                for (left_child, right_child) in children {
                    let a_left_child = Arc::new(left_child.clone());
                    let a_right_child = Arc::new(right_child.clone());
    
                    if check_and(&left_child, &right_child) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::And,
                            left_child: a_left_child.clone(),
                            right_child: a_right_child.clone(),
                        });
                    }
    
                    if check_or(&left_child, &right_child) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::Or,
                            left_child: a_left_child.clone(),
                            right_child: a_right_child.clone(),
                        });
                    }
    
                    if check_implies(&left_child, &right_child) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::Implies,
                            left_child: a_left_child.clone(),
                            right_child: a_right_child.clone(),
                        });
                    }
    
                    if check_until(&left_child, &right_child) {
                        trees.push(SyntaxTree::Binary {
                            op: BinaryOp::Until,
                            left_child: a_left_child,
                            right_child: a_right_child,
                        });
                    }
                }

                trees
            }
        }
    }
    

    // fn depth(&self) -> u8 {
    //     match self {
    //         SkeletonTree::Zeroary => 1,
    //         SkeletonTree::Unary(child) => child.depth() + 1,
    //         SkeletonTree::Binary((left_child, right_child)) => {
    //             left_child.depth().max(right_child.depth()) + 1
    //         }
    //     }
    // }
}

pub fn brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<SyntaxTree> {
    (0..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| gen_formulae::<N>(&skeleton))
            .find(|formula| sample.is_consistent(formula))
    })
}

pub fn mmztn_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<Arc<SyntaxTree>> {
    use rayon::prelude::*;

    let mut formulas: Vec<Vec<Arc<SyntaxTree>>> = Vec::new();

    (0..).into_iter().find_map(|size| {
        if log {
            print!("Generating formulae of size {}", size);
        }
        gen_mmztn_formulae::<N>(&mut formulas);
        let formulae_to_search = formulas.get(size).expect("formulas of given size");
        if log {
            println!(": found {} formulae", formulae_to_search.len());
            println!("Searching formulae of size {}", size);
        }

        formulae_to_search
            // .into_iter()
            // .find(|formula| sample.is_consistent(formula))
            .par_iter()
            .find_any(|formula| sample.is_consistent(formula))
            .cloned()
    })
}

fn gen_mmztn_formulae<const N: usize>(formulas: &mut Vec<Vec<Arc<SyntaxTree>>>) {
    let size = formulas.len();
    let mut new_formulas: Vec<Arc<SyntaxTree>> = Vec::new();
    if size == 0 {
        new_formulas = (0..N)
            .map(|n| {
                Arc::new(SyntaxTree::Zeroary {
                    op: ZeroaryOp::AtomicProp(n as Var),
                })
            })
            .collect::<Vec<Arc<SyntaxTree>>>();
        new_formulas.push(Arc::new(SyntaxTree::Zeroary {
            op: ZeroaryOp::False,
        }));
    } else {
        // Add formulas with unary root
        for child in formulas.get(size - 1).expect("formulas of smaller size") {
            if check_not(&child) {
                new_formulas.push(Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Not,
                    child: child.clone(),
                }));
            }

            if check_next(&child) {
                new_formulas.push(Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Next,
                    child: child.clone(),
                }));
            }

            if check_globally(&child) {
                new_formulas.push(Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Globally,
                    child: child.clone(),
                }));
            }

            if check_finally(&child) {
                new_formulas.push(Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Finally,
                    child: child.clone(),
                }));
            }
        }

        // Add formulas with binary root
        for l_child_size in 0..size {
            let r_child_size = size - 1 - l_child_size;

            for left_child in formulas.get(l_child_size).expect("smaller left formulae") {
                for right_child in formulas.get(r_child_size).expect("smaller left formulae") {
                    if check_and(&left_child, &right_child) {
                        new_formulas.push(Arc::new(SyntaxTree::Binary {
                            op: BinaryOp::And,
                            left_child: left_child.clone(),
                            right_child: right_child.clone(),
                        }));
                    }

                    if check_or(&left_child, &right_child) {
                        new_formulas.push(Arc::new(SyntaxTree::Binary {
                            op: BinaryOp::Or,
                            left_child: left_child.clone(),
                            right_child: right_child.clone(),
                        }));
                    }

                    if check_implies(&left_child, &right_child) {
                        new_formulas.push(Arc::new(SyntaxTree::Binary {
                            op: BinaryOp::Implies,
                            left_child: left_child.clone(),
                            right_child: right_child.clone(),
                        }));
                    }

                    if check_until(&left_child, &right_child) {
                        new_formulas.push(Arc::new(SyntaxTree::Binary {
                            op: BinaryOp::Until,
                            left_child: left_child.clone(),
                            right_child: right_child.clone(),
                        }));
                    }
                }
            }
        }
    }

    formulas.push(new_formulas);
}

pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<SyntaxTree> {
    use rayon::prelude::*;

    (0..).into_iter().find_map(|size| {
        if log {
            println!("Generating formulae of size {}", size);
        }
        // let mut trees = gen_skeleton_trees(size)
        //     .into_iter()
        //     .flat_map(|skeleton| gen_formulae(N, sample.time_lenght(), &skeleton));
        // // if log {
        // //     println!(": found {}", trees.clone().count());
        // // }
        // if log {
        //     println!("Searching formulae of size {}", size);
        // }

        gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| skeleton.gen_formulae::<N>())
            .par_bridge()
            .find_any(|formula| sample.is_consistent(formula))

        // gen_skeleton_trees(size)
        //     .into_par_iter()
        //     .find_map_any(|skeleton|
        //         gen_formulae(N, sample.time_lenght(), &skeleton)
        //             .iter()
        //             .find(|formula| sample.is_consistent(formula))
        //             .map(|formula| format!("{}", formula))
        //     )
    })
}

// Should be possible to compute skeleton trees at compile time
fn gen_skeleton_trees(size: usize) -> Vec<Arc<SkeletonTree>> {
    if size == 0 {
        vec![Arc::new(SkeletonTree::Zeroary)]
    } else {
        let smaller_skeletons = gen_skeleton_trees(size - 1);
        let mut skeletons: Vec<Arc<SkeletonTree>> = smaller_skeletons
            .iter()
            .map(|child| Arc::new(SkeletonTree::Unary(child.clone())))
            .collect();
        for left_size in 0..size {
            let left_smaller_skeletons = gen_skeleton_trees(left_size);
            let right_smaller_skeletons = gen_skeleton_trees(size - 1 - left_size);

            skeletons.extend(
                left_smaller_skeletons
                    .iter()
                    .cartesian_product(right_smaller_skeletons.iter())
                    .map(|(left_child, right_child)| {
                        Arc::new(SkeletonTree::Binary((
                            left_child.clone(),
                            right_child.clone(),
                        )))
                    }),
            );
        }
        skeletons
    }
}

fn gen_formulae<const N: usize>(skeleton: &SkeletonTree) -> Vec<SyntaxTree> {
    match skeleton {
        SkeletonTree::Zeroary => {
            let mut trees = (0..N)
                .map(|n| {
                    SyntaxTree::Zeroary {
                        op: ZeroaryOp::AtomicProp(n as Var),
                    }
                })
                .collect::<Vec<SyntaxTree>>();
            trees.push(SyntaxTree::Zeroary {
                op: ZeroaryOp::False,
            });
            trees
        }
        SkeletonTree::Unary(child) => {
            let mut trees = Vec::new();
            let children = gen_formulae::<N>(child);

            for child in children {
                let a_child = Arc::new(child.clone());

                if check_globally(&child) {
                    trees.push(SyntaxTree::Unary {
                        op: UnaryOp::Globally,
                        child: a_child.clone(),
                    });
                }
                if check_finally(&child) {
                    trees.push(SyntaxTree::Unary {
                        op: UnaryOp::Finally,
                        child: a_child.clone(),
                    });
                }

                if check_not(&child) {
                    trees.push(SyntaxTree::Unary {
                        op: UnaryOp::Not,
                        child: a_child.clone(),
                    });
                }

                if check_next(&child) {
                    trees.push(SyntaxTree::Unary {
                        op: UnaryOp::Next,
                        child: a_child,
                    });
                }

                // trees.extend(
                //     [
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Not,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Next,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Globally,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Finally,
                //             child: child.clone(),
                //         }),
                //     ].into_iter()
                // );
            }

            // trees.extend(children.clone().into_iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Not,
            //         child,
            //     })
            // }));
            // trees.extend(children.clone().into_iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Next,
            //         child,
            //     })
            // }));
            // trees.extend(children.iter().filter(|child| check_globally(child)).map(|child| {
            // // trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Globally,
            //         child: child.clone(),
            //     })
            // }));
            // trees.extend(children.iter().filter(|child| check_finally(child)).map(|child| {
            // // trees.extend(children.iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Finally,
            //         child: child.clone(),
            //     })
            // }));

            // for time in 0..time_lenght {
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::GloballyLeq(time),
            //             child: child.clone(),
            //         })
            //     }));
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::GloballyGneq(time),
            //             child: child.clone(),
            //         })
            //     }));
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::FinallyLeq(time),
            //             child: child.clone(),
            //         })
            //     }));
            // }
            trees
        }
        SkeletonTree::Binary(child) => {
            let mut trees = Vec::new();
            let left_children = gen_formulae::<N>(&child.0);
            let right_children = gen_formulae::<N>(&child.1);
            let children = left_children
                .into_iter()
                .cartesian_product(right_children.into_iter());

            for (left_child, right_child) in children {
                let a_left_child = Arc::new(left_child.clone());
                let a_right_child = Arc::new(right_child.clone());

                if check_and(&left_child, &right_child) {
                    trees.push(SyntaxTree::Binary {
                        op: BinaryOp::And,
                        left_child: a_left_child.clone(),
                        right_child: a_right_child.clone(),
                    });
                }

                if check_or(&left_child, &right_child) {
                    trees.push(SyntaxTree::Binary {
                        op: BinaryOp::Or,
                        left_child: a_left_child.clone(),
                        right_child: a_right_child.clone(),
                    });
                }

                if check_implies(&left_child, &right_child) {
                    trees.push(SyntaxTree::Binary {
                        op: BinaryOp::Implies,
                        left_child: a_left_child.clone(),
                        right_child: a_right_child.clone(),
                    });
                }

                if check_until(&left_child, &right_child) {
                    trees.push(SyntaxTree::Binary {
                        op: BinaryOp::Until,
                        left_child: a_left_child.clone(),
                        right_child: a_right_child.clone(),
                    });
                }
            }

            // // Optimization for symmetric operators: use ordering on syntax trees to cut down the possible trees
            // trees.extend(children.clone().filter_map(|(left_child, right_child)| {
            //     if check_and(&left_child, &right_child) {
            //         Some(
            //             Arc::new(SyntaxTree::Binary {
            //                 op: BinaryOp::And,
            //                 left_child,
            //                 right_child,
            //             })
            //         )
            //     } else if left_child > right_child {
            //         Some(
            //             Arc::new(SyntaxTree::Binary {
            //                 op: BinaryOp::Or,
            //                 left_child,
            //                 right_child,
            //             })
            //         )
            //     } else {
            //         None
            //     }
            // }));

            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Implies,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Until,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Release,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // for time in 0..time_lenght {
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::ReleaseGneq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::ReleaseLeq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::UntillLeq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            // }
            trees
        }
    }
}

fn check_not(child: &SyntaxTree) -> bool {
    match *child {
        // ¬¬φ ≡ φ
        SyntaxTree::Unary { op: UnaryOp::Not, .. }
        // ¬(φ -> ψ) ≡ φ ∧ ¬ψ
        | SyntaxTree::Binary { op: BinaryOp::Implies, .. } => false,
        _ => true,
    }
}

fn check_next(child: &SyntaxTree) -> bool {
    match *child {
        // ¬ X φ ≡ X ¬ φ
        SyntaxTree::Unary {
            op: UnaryOp::Next, ..
        } => false,
        _ => true,
    }
}

fn check_globally(child: &SyntaxTree) -> bool {
    match *child {
        // G G φ <=> G φ
        SyntaxTree::Unary { op: UnaryOp::Globally, .. }
        // ¬ F φ ≡ G ¬ φ
        | SyntaxTree::Unary { op: UnaryOp::Finally, .. } => false,
        _ => true,
    }
}

fn check_finally(child: &SyntaxTree) -> bool {
    match *child {
        // F F φ <=> F φ
        SyntaxTree::Unary {
            op: UnaryOp::Finally,
            ..
        } => false,
        _ => true,
    }
}

fn check_and(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    // Commutative law
    left_child < right_child
        && match (left_child, right_child) {
        // Domination law
        (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (SyntaxTree::Binary { op: BinaryOp::And, .. }, ..)
        // De Morgan's laws
        | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
        // X (φ ∧ ψ) ≡ (X φ) ∧ (X ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
        // G (φ ∧ ψ)≡ (G φ) ∧ (G ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Globally, .. }, SyntaxTree::Unary { op: UnaryOp::Globally, .. }) => false,
        // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
        // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
        (SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *r_1 == *r_2 => false,
        // (φ_1 U ψ) ∧ (φ_2 U ψ) ≡ (φ_1 ∧ φ_2) U ψleft_child: l_1
        (SyntaxTree::Binary { op: BinaryOp::Until, right_child: r_1, .. }, SyntaxTree::Binary { op: BinaryOp::Until, right_child: r_2, .. }) if *r_1 == *r_2 => false,
        // Absorption laws
        (SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }, right_child) if *(l_1.as_ref()) == *right_child || *(r_1.as_ref()) == *right_child => false,
        (left_child, SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }) if *(l_1.as_ref()) == *left_child || *(r_1.as_ref()) == *left_child => false,
        // Distributive laws
        (SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *l_1 == *r_2 || *r_1 == *l_2 || *r_1 == *r_2 => false,
        // G φ ≡ φ ∧ X(G φ)
        (
            left_child,
            SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            }
        ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
            child.as_ref() != left_child
        } else {
            true
        },
        // G φ ≡ X(G φ) ∧ φ
        (
            SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            },
            right_child,
        ) => if let SyntaxTree::Unary { op: UnaryOp::Globally, child } = child.as_ref() {
            child.as_ref() != right_child
        } else {
            true
        },
        _ => true,
    }
}

fn check_or(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    // Commutative law
    left_child < right_child
        && match (left_child, right_child) {
        // Identity law
        (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (SyntaxTree::Binary { op: BinaryOp::Or, .. }, ..)
        // // De Morgan's laws
        // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
        // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
        | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, ..)
        // X (φ ∨ ψ) ≡ (X φ) ∨ (X ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
        // F (φ ∨ ψ) ≡ (F φ) ∨ (F ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Finally, .. }, SyntaxTree::Unary { op: UnaryOp::Finally, .. }) => false,
        // (φ -> ψ_1) ∨ (φ -> ψ_2) ≡ φ -> (ψ_1 ∨ ψ_2)
        // (φ_1 -> ψ) ∨ (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
        (SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_2, right_child: r_2 }) if l_1 == l_2 || r_1 == r_2 => false,
        // (φ U ψ_1) ∨ (φ U ψ_2) ≡ φ U (ψ_1 ∨ ψ_2)
        (SyntaxTree::Binary { op: BinaryOp::Until, left_child: l_1, .. }, SyntaxTree::Binary { op: BinaryOp::Until, left_child: l_2, .. }) if l_1 == l_2 => false,
        // Absorption laws
        (SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }, right_child) if l_1.as_ref() == right_child || r_1.as_ref() == right_child => false,
        (left_child, SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }) if l_1.as_ref() == left_child || r_1.as_ref() == left_child => false,
        // Distributive laws
        (SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::And, left_child: l_2, right_child: r_2 }) if l_1 == l_2 || l_1 == r_2 || r_1 == l_2 || r_1 == r_2 => false,
        // F φ ≡ φ ∨ X(F φ)
        (
            left_child,
            SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            }
        ) => if let SyntaxTree::Unary { op: UnaryOp::Finally, child } = child.as_ref() {
            child.as_ref() != left_child
        } else {
            true
        },
        // F φ ≡ X(F φ) ∨ φ
        (
            SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            },
            right_child,
        ) => if let SyntaxTree::Unary { op: UnaryOp::Finally, child } = child.as_ref() {
            child.as_ref() != right_child
        } else {
            true
        },
        // φ U ψ ≡ ψ ∨ ( φ ∧ X(φ U ψ) )
        // φ U ψ ≡ ψ ∨ ( X(φ U ψ) ∧ φ )
        (
            left_child,
            SyntaxTree::Binary {
                op: BinaryOp::And,
                left_child: l_1,
                right_child: r_1,
            }
        ) => if let SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            } = r_1.as_ref() {
                if let SyntaxTree::Binary {
                    op: BinaryOp::Until,
                    left_child: l_2,
                    right_child: r_2,
                } = child.as_ref() {
                    !(left_child == r_2.as_ref() && l_1 == l_2)
            } else if let SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            } = l_1.as_ref() {
                if let SyntaxTree::Binary {
                    op: BinaryOp::Until,
                    left_child: l_2,
                    right_child: r_2,
                } = child.as_ref() {
                    !(left_child == r_2.as_ref() && r_1 == l_2)
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
            SyntaxTree::Binary {
                op: BinaryOp::And,
                left_child: l_1,
                right_child: r_1,
            },
            right_child
        ) => if let SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            } = r_1.as_ref() {
                if let SyntaxTree::Binary {
                    op: BinaryOp::Until,
                    left_child: l_2,
                    right_child: r_2,
                } = child.as_ref() {
                    !(right_child == r_2.as_ref() && l_1 == l_2)
            // // Made useless by commutativity optimization on ∧
            } else if let SyntaxTree::Unary {
                op: UnaryOp::Next,
                child,
            } = l_1.as_ref() {
                if let SyntaxTree::Binary {
                    op: BinaryOp::Until,
                    left_child: l_2,
                    right_child: r_2,
                } = child.as_ref() {
                    !(right_child == r_2.as_ref() && r_1 == l_2)
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

fn check_implies(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    match (left_child, right_child) {
        // Ex falso quodlibet (need to define True)
        // (SyntaxTree::Zeroary { op: ZeroaryOp::False, .. }, ..)
        // // φ -> ψ ≡ ¬ψ -> ¬φ
        // (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
        // ¬φ -> ψ ≡ ψ ∨ φ
        (
            SyntaxTree::Unary {
                op: UnaryOp::Not, ..
            },
            ..,
        ) => false,
        _ => true,
    }
}

fn check_until(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    match (left_child, right_child) {
        // X (φ U ψ) ≡ (X φ) U (X ψ)
        (
            SyntaxTree::Unary {
                op: UnaryOp::Next, ..
            },
            SyntaxTree::Unary {
                op: UnaryOp::Next, ..
            },
        ) => false,
        (
            left_child,
            SyntaxTree::Binary {
                op: BinaryOp::Until,
                left_child: l_1,
                ..
            },
        ) if left_child == l_1.as_ref() => false,
        _ => true,
    }
}

// fn solve_skeleton(skeleton: &SkeletonTree) {
//     use z3::*;

//     let mut cfg = Config::new();
//     cfg.set_model_generation(true);
//     let ctx = Context::new(&cfg);
//     let solver = Solver::new(&ctx);

//     solver.assert(&Bool::and(
//         &ctx,
//         &[
//             &Bool::new_const(&ctx, 0).not(),
//             &Bool::new_const(&ctx, 1).not(),
//         ],
//     ));
//     if let SatResult::Sat = solver.check() {
//         if let Some(model) = solver.get_model() {
//             if let Some(x_0) = model.eval(&Bool::new_const(&ctx, 0), false) {
//                 println!("{}", x_0.as_bool().expect("Boolean value"));
//             }
//             if let Some(x_1) = model.eval(&Bool::new_const(&ctx, 1), false) {
//                 println!("{}", x_1.as_bool().expect("Boolean value"));
//             }
//         }
//     }
// }
