//! Target machine descriptions — the LLVM TargetMachine equivalent.

/// A target triple (`arch-vendor-os-env`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    pub arch: String,
    pub os: String,
}

impl Triple {
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.split('-');
        let arch = it.next()?.to_string();
        let os = it.nth(1).unwrap_or("unknown").to_string();
        if arch.is_empty() { return None; }
        Some(Self { arch, os })
    }
}

/// Code-generation features a target machine advertises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetMachine {
    pub triple: Triple,
    pub has_simd: bool,
    pub has_atomic: bool,
    pub pointer_width: u32,
}

impl TargetMachine {
    /// Built-in target table — the LLVM `llvm::TargetRegistry` subset.
    pub fn lookup(triple_str: &str) -> Option<Self> {
        let triple = Triple::parse(triple_str)?;
        let (simd, atomic, width) = match triple.arch.as_str() {
            "x86_64" => (true, true, 64),
            "aarch64" | "arm64" => (true, true, 64),
            "riscv64" => (true, true, 64),
            "i386" | "i686" => (true, true, 32),
            "arm" => (true, false, 32),
            _ => return None,
        };
        Some(Self { triple, has_simd: simd, has_atomic: atomic, pointer_width: width })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_targets() {
        let t = TargetMachine::lookup("x86_64-unknown-linux").unwrap();
        assert_eq!(t.pointer_width, 64);
        assert!(t.has_atomic);
        assert_eq!(TargetMachine::lookup("riscv64-unknown-elf").unwrap().pointer_width, 64);
        assert!(TargetMachine::lookup("mips-unknown").is_none());
    }
}
