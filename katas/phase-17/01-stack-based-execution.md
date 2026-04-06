---
id: stack-based-execution
phase: 17
phase_title: "What is WebAssembly Really?"
sequence: 1
title: Stack-Based Execution — How a Virtual Machine Evaluates
hints:
  - WASM instructions push and pop values from an implicit operand stack
  - What happens if you pop from an empty stack?
  - The WASM validator checks stack depth at load time, before any code runs
  - Use Result to handle underflow instead of panicking
---

## Description

WebAssembly is a stack-based virtual machine. Instructions do not name their operands -- instead, they push results onto and pop arguments from an implicit value stack. For example, to compute `3 + 4`, a WASM program pushes `3`, pushes `4`, then executes `i32.add`, which pops two values and pushes their sum. If an instruction tries to pop from an empty stack, the program has a bug. Real WASM runtimes catch this at validation time (before execution). This kata simulates a stack-based VM in Rust and demonstrates what happens when stack depth is not checked.

## Broken Code

```rust
#[derive(Debug)]
enum Instruction {
    Push(i64),
    Add,
    Mul,
}

fn execute(program: &[Instruction]) -> i64 {
    let mut stack: Vec<i64> = Vec::new();

    for instr in program {
        match instr {
            Instruction::Push(val) => stack.push(*val),
            Instruction::Add => {
                // BUG: No check for stack underflow.
                // If the stack has fewer than 2 values, this panics.
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a + b);
            }
            Instruction::Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a * b);
            }
        }
    }

    stack.pop().unwrap()
}

fn main() {
    // Valid program: Push(3), Push(4), Add => 7
    let valid = vec![
        Instruction::Push(3),
        Instruction::Push(4),
        Instruction::Add,
    ];
    println!("Result: {}", execute(&valid));

    // Invalid program: Add without enough operands on the stack
    let invalid = vec![
        Instruction::Push(5),
        Instruction::Add, // Only one value on stack -- underflow!
    ];
    println!("Result: {}", execute(&invalid));
}
```

## Correct Code

```rust
#[derive(Debug)]
enum Instruction {
    Push(i64),
    Add,
    Mul,
}

#[derive(Debug)]
enum VmError {
    StackUnderflow { instruction: &'static str, needed: usize, had: usize },
    StackEmpty,
}

fn execute(program: &[Instruction]) -> Result<i64, VmError> {
    let mut stack: Vec<i64> = Vec::new();

    for instr in program {
        match instr {
            Instruction::Push(val) => stack.push(*val),
            Instruction::Add => {
                // Correct: check stack depth before popping
                if stack.len() < 2 {
                    return Err(VmError::StackUnderflow {
                        instruction: "Add",
                        needed: 2,
                        had: stack.len(),
                    });
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a + b);
            }
            Instruction::Mul => {
                if stack.len() < 2 {
                    return Err(VmError::StackUnderflow {
                        instruction: "Mul",
                        needed: 2,
                        had: stack.len(),
                    });
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a * b);
            }
        }
    }

    stack.pop().ok_or(VmError::StackEmpty)
}

fn main() {
    let valid = vec![
        Instruction::Push(3),
        Instruction::Push(4),
        Instruction::Add,
    ];
    match execute(&valid) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {:?}", e),
    }

    let invalid = vec![
        Instruction::Push(5),
        Instruction::Add,
    ];
    match execute(&invalid) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Explanation

The broken version calls `stack.pop().unwrap()` without checking whether the stack has enough values. When the `Add` instruction runs with only one value on the stack, the second `pop()` returns `None`, and `unwrap()` panics.

**How real WASM handles this:**

WASM uses a *validation phase* that runs before any code executes. The validator walks through every instruction and tracks the stack depth statically. If any instruction would underflow the stack, the module is rejected at load time -- the code never runs. This means a valid WASM module can never experience a stack underflow at runtime.

**Why stack-based?**

Stack-based VMs have a key advantage: instructions are compact. An `i32.add` instruction is a single byte -- it does not need to encode register names or operand addresses. This makes WASM binaries small and fast to decode, which is critical for web delivery where download size matters.

**The Rust simulation:**

Our `VmError` enum models what the WASM validator would catch. In real WASM, this check happens once at load time (O(n) in the number of instructions). In our simulation, we check at runtime per-instruction, which is slightly different but teaches the same principle: never assume the stack has values without verification.

The invariant violated in the broken code: **every instruction that consumes stack values must verify the stack has enough values before popping.**

## ⚠️ Caution

- Using `unwrap()` on stack pops in a real VM implementation masks the real error — stack underflow. Always validate stack depth before operations.
- WASM validation happens before execution. A module with invalid stack usage will be rejected at load time, not at runtime.

## 💡 Tips

- Think of WASM instructions as consuming inputs from the stack and pushing results. `i32.add` pops two values and pushes one.
- Use Rust enums to model WASM instructions for type-safe interpretation.

## Compiler Error Interpretation

```
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value',
  src/main.rs:20:42
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic, not a compiler error. The code compiles because `unwrap()` is a valid method on `Option<i64>`. The problem is a logic error: the code assumes the stack always has values to pop.

In the error:

1. **"called `Option::unwrap()` on a `None` value"** -- `stack.pop()` returned `None` because the stack was empty. The `unwrap()` panicked because it had no value to return.
2. **"src/main.rs:20:42"** -- the panic occurred on the second `stack.pop().unwrap()` inside the `Add` arm, which is the first pop that runs out of values.

The deeper lesson: in WASM, this class of bug is caught *before* execution, during validation. The type system of a real WASM runtime is strict enough to prevent this structurally. Our `Result`-based fix catches it at runtime, which is the next best thing when we cannot do static validation.

---

| [Prev: Type-State Pattern — Encoding State in the Type System](#/katas/type-state-pattern) | [Next: Linear Memory — Bounds-Checked Byte Arrays](#/katas/linear-memory-bounds) |
