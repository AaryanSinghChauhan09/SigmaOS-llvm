//! LLVM-style intermediate representation.

use std::collections::BTreeMap;
use std::fmt;

/// IR value type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Void,
    Bool,
    Int(u32),
    Float,
    Ptr,
    /// Named struct type.
    Struct(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "i1"),
            Type::Int(n) => write!(f, "i{n}"),
            Type::Float => write!(f, "float"),
            Type::Ptr => write!(f, "ptr"),
            Type::Struct(n) => write!(f, "%{n}"),
        }
    }
}

/// A named IR value (register-like).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub name: String,
    pub ty: Type,
}

/// Operand of an instruction — either a named value or a constant.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Value(String),
    ConstI64(i64),
    ConstBool(bool),
}

/// IR instruction kinds (the subset a pass pipeline needs).
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// `%r = add i64 %a, %b`
    BinaryOp { op: &'static str, dst: String, lhs: Operand, rhs: Operand, ty: Type },
    /// `%r = icmp eq i64 %a, %b`
    Icmp { pred: &'static str, dst: String, lhs: Operand, rhs: Operand, ty: Type },
    /// `br i1 %c, label %t, label %f`
    Branch { cond: Operand, then_bb: String, else_bb: String },
    /// `ret void` / `ret i64 %v`
    Ret { val: Option<Operand> },
    /// `call void @f(...)`
    Call { func: String, args: Vec<Operand> },
    /// `store i64 %v, ptr %p`
    Store { val: Operand, ptr: Operand },
    /// `load i64, ptr %p`
    Load { dst: String, ptr: Operand, ty: Type },
}

impl Instruction {
    /// Names this instruction defines, if any.
    pub fn defined_names(&self) -> Vec<String> {
        match self {
            Instruction::BinaryOp { dst, .. }
            | Instruction::Icmp { dst, .. }
            | Instruction::Load { dst, .. } => vec![dst.clone()],
            _ => vec![],
        }
    }

    /// Value names this instruction uses.
    pub fn used_names(&self) -> Vec<String> {
        let ops: Vec<&Operand> = match self {
            Instruction::BinaryOp { lhs, rhs, .. } | Instruction::Icmp { lhs, rhs, .. } => {
                vec![lhs, rhs]
            }
            Instruction::Branch { cond, .. } => vec![cond],
            Instruction::Ret { val } => val.iter().collect(),
            Instruction::Call { args, .. } => args.iter().collect(),
            Instruction::Store { val, ptr } => vec![val, ptr],
            Instruction::Load { ptr, .. } => vec![ptr],
        };
        ops.into_iter()
            .filter_map(|o| match o {
                Operand::Value(n) => Some(n.clone()),
                _ => None,
            })
            .collect()
    }
}

/// A basic block: label + instruction list, terminated by branch/ret.
#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    pub label: String,
    pub insts: Vec<Instruction>,
}

impl BasicBlock {
    /// Terminator (last instruction), if it is one.
    pub fn terminator(&self) -> Option<&Instruction> {
        match self.insts.last() {
            Some(Instruction::Branch { .. }) | Some(Instruction::Ret { .. }) => self.insts.last(),
            _ => None,
        }
    }
}

/// A function: name, signature, entry block + body blocks.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub ret_ty: Type,
    pub params: Vec<Value>,
    pub blocks: Vec<BasicBlock>,
}

impl Function {
    /// Entry block label.
    pub fn entry(&self) -> &str {
        self.blocks.first().map(|b| b.label.as_str()).unwrap_or("")
    }
}

/// An LLVM module: named functions + named global values.
#[derive(Debug, Default)]
pub struct Module {
    pub functions: BTreeMap<String, Function>,
    pub globals: BTreeMap<String, Type>,
}

impl Module {
    pub fn new() -> Self { Self::default() }

    pub fn add_function(&mut self, f: Function) {
        self.functions.insert(f.name.clone(), f);
    }

    /// All value names defined across the module.
    pub fn defined_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        for f in self.functions.values() {
            for b in &f.blocks {
                for i in &b.insts {
                    out.extend(i.defined_names());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_module() {
        let mut m = Module::new();
        m.add_function(Function {
            name: "main".into(),
            ret_ty: Type::Int(32),
            params: vec![],
            blocks: vec![BasicBlock {
                label: "entry".into(),
                insts: vec![
                    Instruction::Ret { val: Some(Operand::ConstI64(0)) },
                ],
            }],
        });
        assert_eq!(m.functions["main"].entry(), "entry");
        assert!(m.functions["main"].blocks[0].terminator().is_some());
    }
}
