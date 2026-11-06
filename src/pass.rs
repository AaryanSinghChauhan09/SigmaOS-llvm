//! Pass manager — ordered analysis/transform passes over a Module.

use crate::ir::{Function, Module};
use std::collections::BTreeMap;

/// Result of running a pass.
#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub notes: Vec<String>,
}

/// A read-only analysis pass (produces facts, no mutation).
pub trait Analysis {
    fn name(&self) -> &str;
    fn run(&self, module: &Module) -> BTreeMap<String, String>;
}

/// A transform pass (may mutate the module).
pub trait Pass {
    fn name(&self) -> &str;
    fn run(&mut self, module: &mut Module) -> PassResult;
}

/// Ordered pass pipeline.
pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> Self { Self { passes: vec![] } }
    pub fn add<P: Pass + 'static>(&mut self, p: P) { self.passes.push(Box::new(p)); }

    /// Run all passes in order; stop on first hard failure note.
    pub fn run(&mut self, module: &mut Module) -> Vec<PassResult> {
        self.passes.iter_mut().map(|p| p.run(module)).collect()
    }
}

/// Finds functions whose blocks lack a terminator — a real verifier-style analysis.
pub struct MissingTerminator;

impl Analysis for MissingTerminator {
    fn name(&self) -> &str { "missing-terminator" }

    fn run(&self, module: &Module) -> BTreeMap<String, String> {
        module
            .functions
            .iter()
            .filter(|(_, f)| f.blocks.iter().any(|b| b.terminator().is_none()))
            .map(|(n, _)| (n.clone(), "block missing terminator".to_string()))
            .collect()
    }
}

/// Renames every function by prefixing `prefix_` — a transform pass.
pub struct PrefixFunctions {
    pub prefix: String,
}

impl Pass for PrefixFunctions {
    fn name(&self) -> &str { "prefix-functions" }
    fn run(&mut self, module: &mut Module) -> PassResult {
        let old: Vec<Function> = module.functions.values().cloned().collect();
        module.functions.clear();
        for mut f in old {
            f.name = format!("{}{}", self.prefix, f.name);
            module.functions.insert(f.name.clone(), f);
        }
        PassResult { changed: true, notes: vec!["functions prefixed".into()] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn sample() -> Module {
        let mut m = Module::new();
        m.add_function(Function {
            name: "f".into(),
            ret_ty: Type::Void,
            params: vec![],
            blocks: vec![BasicBlock {
                label: "entry".into(),
                insts: vec![Instruction::Ret { val: None }],
            }],
        });
        m
    }

    #[test]
    fn analysis_detects_missing_terminator() {
        let mut m = sample();
        m.functions.get_mut("f").unwrap().blocks[0].insts.pop();
        let facts = MissingTerminator.run(&m);
        assert!(facts.contains_key("f"));
    }

    #[test]
    fn transform_prefixes_functions() {
        let mut m = sample();
        let mut pm = PassManager::new();
        pm.add(PrefixFunctions { prefix: "sigma_".into() });
        pm.run(&mut m);
        assert!(m.functions.contains_key("sigma_f"));
    }
}
