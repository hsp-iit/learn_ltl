// use z3::ast::Bool;

use crate::syntax::*;
use crate::trace::*;

// pub fn learn<const N: usize>(sample: Sample<N>) -> Option<SyntaxTree> {
//     unimplemented!();
// }

// pub fn learn_size<const N: usize>(sample: Sample<N>, size: usize) -> Option<SyntaxTree> {
//     unimplemented!();
// }

#[derive(Debug, Clone)]
enum SkeletonTree {
    Zeroary,
    Unary(Box<SkeletonTree>),
    Binary(Box<(SkeletonTree, SkeletonTree)>),
}

impl SkeletonTree {
    fn depth(&self) -> u8 {
        match self {
            SkeletonTree::Zeroary => 1,
            SkeletonTree::Unary(child) => child.depth() + 1,
            SkeletonTree::Binary(children) => children.0.depth().max(children.1.depth()) + 1,
        }
    }
}

pub fn brute_solve<const N: usize>(sample: &Sample<N>) -> Option<SyntaxTree> {
    gen_skeleton_trees(5).iter().find_map(|skeleton| {
        gen_formulae(N, sample.time_lenght(), skeleton)
            .into_iter()
            .find(|formula| sample.is_consistent(formula))
    })
}

// Should be possible to compute skeleton trees at compile time
fn gen_skeleton_trees(max_depth: u8) -> Vec<SkeletonTree> {
    use itertools::Itertools;

    let mut skeletons = vec![SkeletonTree::Zeroary];
    if max_depth > 1 {
        let smaller_skeletons = gen_skeleton_trees(max_depth - 1);
        skeletons.extend(
            smaller_skeletons
                .iter()
                .map(|child| SkeletonTree::Unary(Box::new(child.clone()))),
        );
        skeletons.extend(
            smaller_skeletons
                .iter()
                .cartesian_product(&smaller_skeletons)
                .map(|children| {
                    SkeletonTree::Binary(Box::new((children.0.clone(), children.1.clone())))
                }),
        );
    }
    skeletons
}

fn gen_formulae(atomics: usize, time_lenght: u8, skeleton: &SkeletonTree) -> Vec<SyntaxTree> {
    use itertools::Itertools;

    match skeleton {
        SkeletonTree::Zeroary => {
            let mut trees = (0..atomics)
                .map(|n| SyntaxTree::Zeroary {
                    op: ZeroaryOp::AtomicProp(n),
                })
                .collect::<Vec<SyntaxTree>>();
            trees.push(SyntaxTree::Zeroary {
                op: ZeroaryOp::False,
            });
            trees
        }
        SkeletonTree::Unary(child) => {
            let mut trees = Vec::new();
            let children = gen_formulae(atomics, time_lenght, child);
            trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                op: UnaryOp::Not,
                child: Box::new(child.clone()),
            }));
            trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                op: UnaryOp::Next,
                child: Box::new(child.clone()),
            }));
            trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                op: UnaryOp::Globally,
                child: Box::new(child.clone()),
            }));
            for time in 0..time_lenght {
                trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                    op: UnaryOp::GloballyLeq(time),
                    child: Box::new(child.clone()),
                }));
                trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                    op: UnaryOp::GloballyGneq(time),
                    child: Box::new(child.clone()),
                }));
                trees.extend(children.iter().map(|child| SyntaxTree::Unary {
                    op: UnaryOp::FinallyLeq(time),
                    child: Box::new(child.clone()),
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
            trees.extend(
                children
                    .clone()
                    .map(|(left_child, right_child)| SyntaxTree::Binary {
                        op: BinaryOp::And,
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }),
            );
            trees.extend(
                children
                    .clone()
                    .map(|(left_child, right_child)| SyntaxTree::Binary {
                        op: BinaryOp::Or,
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }),
            );
            trees.extend(
                children
                    .clone()
                    .map(|(left_child, right_child)| SyntaxTree::Binary {
                        op: BinaryOp::Release,
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }),
            );
            for time in 0..time_lenght {
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    SyntaxTree::Binary {
                        op: BinaryOp::ReleaseGneq(time),
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }
                }));
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    SyntaxTree::Binary {
                        op: BinaryOp::ReleaseLeq(time),
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }
                }));
                trees.extend(children.clone().map(|(left_child, right_child)| {
                    SyntaxTree::Binary {
                        op: BinaryOp::UntillLeq(time),
                        left_child: Box::new(left_child),
                        right_child: Box::new(right_child),
                    }
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
