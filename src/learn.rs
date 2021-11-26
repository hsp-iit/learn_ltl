// use z3::ast::Bool;

use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;
// use std::Arc::Arc;
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
    fn depth(&self) -> u8 {
        match self {
            SkeletonTree::Zeroary => 1,
            SkeletonTree::Unary(child) => child.depth() + 1,
            SkeletonTree::Binary((left_child, right_child)) => {
                left_child.depth().max(right_child.depth()) + 1
            }
        }
    }
}

pub fn brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<Arc<SyntaxTree>> {
    (0..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| gen_formulae(N, sample.time_lenght(), &skeleton))
            .find(|formula| sample.is_consistent(formula))
    })
}

pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<Arc<SyntaxTree>> {
    use rayon::prelude::*;

    (0..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| gen_formulae(N, sample.time_lenght(), &skeleton))
            .par_bridge()
            .find_any(|formula| sample.is_consistent(formula))
    })
}

// Should be possible to compute skeleton trees at compile time
fn gen_skeleton_trees(size: usize) -> Vec<Arc<SkeletonTree>> {
    let mut skeletons = vec![Arc::new(SkeletonTree::Zeroary)];
    if size > 0 {
        let smaller_skeletons = gen_skeleton_trees(size - 1);
        skeletons.extend(
            smaller_skeletons
                .iter()
                .map(|child| Arc::new(SkeletonTree::Unary(child.clone()))),
        );
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
    }
    skeletons
}

fn gen_formulae(atomics: usize, time_lenght: u8, skeleton: &SkeletonTree) -> Vec<Arc<SyntaxTree>> {
    match skeleton {
        SkeletonTree::Zeroary => {
            let mut trees = (0..atomics)
                .map(|n| {
                    Arc::new(SyntaxTree::Zeroary {
                        op: ZeroaryOp::AtomicProp(n),
                    })
                })
                .collect::<Vec<Arc<SyntaxTree>>>();
            trees.push(Arc::new(SyntaxTree::Zeroary {
                op: ZeroaryOp::False,
            }));
            trees
        }
        SkeletonTree::Unary(child) => {
            let mut trees = Vec::new();
            let children = gen_formulae(atomics, time_lenght, child);
            trees.extend(children.iter().map(|child| {
                Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Not,
                    child: child.clone(),
                })
            }));
            trees.extend(children.iter().map(|child| {
                Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Next,
                    child: child.clone(),
                })
            }));
            trees.extend(children.iter().map(|child| {
                Arc::new(SyntaxTree::Unary {
                    op: UnaryOp::Globally,
                    child: child.clone(),
                })
            }));
            for time in 0..time_lenght {
                trees.extend(children.iter().map(|child| {
                    Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::GloballyLeq(time),
                        child: child.clone(),
                    })
                }));
                trees.extend(children.iter().map(|child| {
                    Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::GloballyGneq(time),
                        child: child.clone(),
                    })
                }));
                trees.extend(children.iter().map(|child| {
                    Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::FinallyLeq(time),
                        child: child.clone(),
                    })
                }));
            }
            trees
        }
        SkeletonTree::Binary(child) => {
            let mut trees = Vec::new();
            let left_children = gen_formulae(atomics, time_lenght, &child.0);
            let right_children = gen_formulae(atomics, time_lenght, &child.1);
            let children = left_children
                .into_iter()
                .cartesian_product(right_children.into_iter());
            trees.extend(children.clone().map(|(left_child, right_child)| {
                Arc::new(SyntaxTree::Binary {
                    op: BinaryOp::And,
                    left_child,
                    right_child,
                })
            }));
            trees.extend(children.clone().map(|(left_child, right_child)| {
                Arc::new(SyntaxTree::Binary {
                    op: BinaryOp::Or,
                    left_child,
                    right_child,
                })
            }));
            trees.extend(children.clone().map(|(left_child, right_child)| {
                Arc::new(SyntaxTree::Binary {
                    op: BinaryOp::Release,
                    left_child,
                    right_child,
                })
            }));
            for time in 0..time_lenght {
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::ReleaseGneq(time),
                        left_child,
                        right_child,
                    })
                }));
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::ReleaseLeq(time),
                        left_child,
                        right_child,
                    })
                }));
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::UntillLeq(time),
                        left_child,
                        right_child,
                    })
                }));
            }
            trees
        }
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
