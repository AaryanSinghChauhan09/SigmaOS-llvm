//! SigmaOS-llvm — compiler infrastructure library.
//!
//! Models the LLVM core: IR (modules, functions, basic blocks,
//! instructions), an analysis/transform pass manager, and target
//! machine descriptions. Deterministic, std-only, no actual codegen.

pub mod ir;
pub mod pass;
pub mod target;

pub use ir::{BasicBlock, Function, Instruction, Module, Operand, Type, Value};
pub use pass::{Analysis, Pass, PassManager, PassResult};
pub use target::{TargetMachine, Triple};
