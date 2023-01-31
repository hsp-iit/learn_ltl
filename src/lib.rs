//! Data structures and tools to passively learn Linear Temporal Logic (LTL) formulas from a sample of positive and negative finite traces
//! (see [Neider, Gavran - Learning Linear Temporal Properties (2018)](https://doi.org/10.23919/FMCAD.2018.8603016)).
//!
//! LTL formulae are represented as recursive trees.
//!
//! ```
//! use learn_ltl::SyntaxTree;
//! use std::sync::Arc;
//!
//! const ATOM_0: SyntaxTree = SyntaxTree::Atom(0);
//!
//! let not = SyntaxTree::Not(Arc::new(ATOM_0));
//! let next = SyntaxTree::Next(Arc::new(ATOM_0));
//! let globally = SyntaxTree::Globally(Arc::new(ATOM_0));
//! let finally = SyntaxTree::Finally(Arc::new(ATOM_0));
//!
//! const ATOM_1: SyntaxTree = SyntaxTree::Atom(1);
//!
//! let and = SyntaxTree::And(Arc::new(ATOM_0), Arc::new(ATOM_1));
//! let or = SyntaxTree::Or(Arc::new(ATOM_0), Arc::new(ATOM_1));
//! let until = SyntaxTree::Until(Arc::new(ATOM_0), Arc::new(ATOM_1));
//! ```
//!
//! [`Trace`]s are defined as `Trace<N> = Vec<[bool; N]>` where `N` is a `const` parameter.
//! A [`SyntaxTree`] can be evaluated over a [`Trace`].
//!
//! ```
//! let tt_trace = vec![[true, true]];
//! assert!(tt_trace.eval(&and));
//!
//! let ff_trace = vec![[false, false]];
//! assert!(!ff_trace.eval(&and));
//! ```
//!
//! A sample is given by two [`Vec`]s of [`Trace`]s, and (optionally) custom variable names.
//!
//! A [`SyntaxTree`] can be evaluated over a [`Sample`].
//! ```
//! use learn_ltl::Sample;
//!
//! let sample = Sample {
//!     var_names: Sample::var_names(),
//!     positive_traces: vec![vec![[true, true]]],
//!     negative_traces: vec![
//!         vec![[false, true]],
//!         vec![[true, false]],
//!         vec![[false, false]],
//!     ],
//! };
//!
//! assert!(sample.is_consistent(&and));
//! assert!(!sample.is_consistent(&and));
//! ```

mod learn;

/// This module contains the definition of
mod syntax;

mod trace;

pub use learn::*;
pub use syntax::*;
pub use trace::*;
