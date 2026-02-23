---
id: sandboxed-script-execution
phase: 25
phase_title: "Capstone Projects"
sequence: 2
title: "Capstone: Sandboxed Script Interpreter with Bounded Resources"
hints:
  - A Print instruction should go through a host callback, not call println! directly
  - The host callback can enforce an output budget (maximum number of outputs)
  - Combine fuel metering (instruction limit) with output budgets for full sandboxing
  - Return partial results when any budget is exhausted, do not panic
---

## Description

This capstone combines the stack-based VM (Phase 17), fuel metering (Phase 24), host callbacks (Phase 21), and capability passing (Phase 23) into a sandboxed script interpreter. User-provided bytecode runs inside a metered sandbox. All I/O goes through host-controlled callbacks with budgets. The interpreter halts cleanly when any resource budget is exhausted, returning partial results. This is the architecture of WASM-based scripting runtimes like Cloudflare Workers and Shopify Functions.

## Broken Code

```rust
#[derive(Debug, Clone)]
enum Op {
    Push(i64),
    Add,
    Dup,
    Print,
    JmpNz(usize),
    Halt,
}

struct Interpreter {
    stack: Vec<i64>,
    output: Vec<String>,
    max_output: usize,
}

impl Interpreter {
    fn new(max_output: usize) -> Self {
        Interpreter {
            stack: Vec::new(),
            output: Vec::new(),
            max_output,
        }
    }

    fn run(&mut self, program: &[Op]) {
        let mut pc = 0;
        while pc < program.len() {
            match &program[pc] {
                Op::Push(v) => self.stack.push(*v),
                Op::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Op::Dup => {
                    let top = *self.stack.last().unwrap();
                    self.stack.push(top);
                }
                Op::Print => {
                    let val = *self.stack.last().unwrap();
                    // BUG: Direct println! -- ambient authority, no budget check.
                    // In WASM, this would bypass the host entirely.
                    println!("Output: {}", val);
                    self.output.push(format!("{}", val));
                }
                Op::JmpNz(target) => {
                    let top = self.stack.pop().unwrap();
                    if top != 0 {
                        pc = *target;
                        continue;
                    }
                }
                Op::Halt => break,
            }
            pc += 1;
        }
    }
}

fn main() {
    let mut interp = Interpreter::new(10);

    // A looping program that prints on every iteration
    let program = vec![
        Op::Push(1),
        Op::Dup,
        Op::Print,
        Op::JmpNz(1),
        Op::Halt,
    ];

    interp.run(&program);

    // Should have stopped at the output budget
    assert!(
        interp.output.len() <= 10,
        "Output budget exceeded: {} outputs (max 10)!",
        interp.output.len()
    );
}
```

## Correct Code

```rust
#[derive(Debug, Clone)]
enum Op {
    Push(i64),
    Add,
    Dup,
    Print,
    JmpNz(usize),
    Halt,
}

#[derive(Debug)]
enum HaltReason {
    Completed,
    OutOfFuel { executed: u64 },
    OutputBudgetExhausted { produced: usize },
    StackUnderflow,
}

/// Host callbacks with budgets. All I/O goes through here.
struct HostCallbacks {
    output: Vec<String>,
    output_budget: usize,
}

impl HostCallbacks {
    fn new(output_budget: usize) -> Self {
        HostCallbacks {
            output: Vec::new(),
            output_budget,
        }
    }

    fn print(&mut self, value: i64) -> Result<(), HaltReason> {
        if self.output.len() >= self.output_budget {
            return Err(HaltReason::OutputBudgetExhausted {
                produced: self.output.len(),
            });
        }
        self.output.push(format!("{}", value));
        Ok(())
    }
}

struct Interpreter {
    stack: Vec<i64>,
    fuel: u64,
    instructions_executed: u64,
    host: HostCallbacks,
}

impl Interpreter {
    fn new(fuel: u64, output_budget: usize) -> Self {
        Interpreter {
            stack: Vec::new(),
            fuel,
            instructions_executed: 0,
            host: HostCallbacks::new(output_budget),
        }
    }

    fn consume_fuel(&mut self) -> Result<(), HaltReason> {
        match self.fuel.checked_sub(1) {
            Some(remaining) => {
                self.fuel = remaining;
                self.instructions_executed += 1;
                Ok(())
            }
            None => Err(HaltReason::OutOfFuel {
                executed: self.instructions_executed,
            }),
        }
    }

    fn run(&mut self, program: &[Op]) -> HaltReason {
        let mut pc = 0;

        while pc < program.len() {
            // Check fuel before each instruction
            if let Err(reason) = self.consume_fuel() {
                return reason;
            }

            match &program[pc] {
                Op::Push(v) => self.stack.push(*v),
                Op::Add => {
                    if self.stack.len() < 2 {
                        return HaltReason::StackUnderflow;
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Op::Dup => {
                    if self.stack.is_empty() {
                        return HaltReason::StackUnderflow;
                    }
                    let top = *self.stack.last().unwrap();
                    self.stack.push(top);
                }
                Op::Print => {
                    if self.stack.is_empty() {
                        return HaltReason::StackUnderflow;
                    }
                    let val = *self.stack.last().unwrap();
                    // All output goes through the host callback with budget enforcement
                    if let Err(reason) = self.host.print(val) {
                        return reason;
                    }
                }
                Op::JmpNz(target) => {
                    if self.stack.is_empty() {
                        return HaltReason::StackUnderflow;
                    }
                    let top = self.stack.pop().unwrap();
                    if top != 0 {
                        pc = *target;
                        continue;
                    }
                }
                Op::Halt => return HaltReason::Completed,
            }
            pc += 1;
        }

        HaltReason::Completed
    }
}

fn main() {
    let mut interp = Interpreter::new(1000, 10);

    let program = vec![
        Op::Push(1),
        Op::Dup,
        Op::Print,
        Op::JmpNz(1),
        Op::Halt,
    ];

    let reason = interp.run(&program);

    println!("Halt reason: {:?}", reason);
    println!("Instructions executed: {}", interp.instructions_executed);
    println!("Output produced: {} values", interp.host.output.len());
    println!("Output: {:?}", interp.host.output);

    assert!(
        interp.host.output.len() <= 10,
        "Output budget exceeded: {} outputs!",
        interp.host.output.len()
    );

    println!("\nSandboxed execution completed safely!");
}
```

## Explanation

The broken version has two critical issues:

1. **No fuel metering:** The infinite loop (`JmpNz(1)` with nonzero stack top) runs forever without any instruction limit.
2. **Direct I/O:** `println!` is called directly instead of going through a budgeted host callback. The `max_output` field exists but is never checked.

The program hangs because the loop never terminates, and the assertion is never reached.

**The complete sandbox model:**

The correct version enforces three independent budgets:

1. **Fuel** (instruction count): limits total computation. Prevents infinite loops.
2. **Output budget**: limits how much data the guest can emit. Prevents log flooding.
3. **Stack depth** (implicit): stack underflow is caught and reported cleanly.

Each budget is checked *before* the action it gates:
- Fuel is consumed before executing each instruction
- Output budget is checked before adding to the output buffer
- Stack depth is checked before popping

**The host callback pattern:**

All I/O goes through `HostCallbacks`. The interpreter never calls `println!` directly. This ensures:
- All output is captured (for logging, testing, or forwarding)
- Output can be budgeted and rate-limited
- The host maintains full control over what the guest can emit

**This capstone integrates:**
- **Phase 17:** Stack-based VM with stack underflow checking
- **Phase 20:** Error handling as return values (HaltReason enum, not panics)
- **Phase 21:** Separation of compute (interpreter) from I/O (host callbacks)
- **Phase 23:** Sandboxed execution with resource limits
- **Phase 24:** Fuel metering for instruction count limits

**Real-world parallels:**
- **Cloudflare Workers:** WASM modules with CPU time limits and I/O quotas
- **Shopify Functions:** 5ms execution limit, bounded output size
- **Fastly Compute:** Per-request resource budgets

The invariant violated in the broken code: **sandboxed interpreters must meter all resources (instructions, I/O, memory) and halt cleanly when any budget is exhausted.**

## Compiler Error Interpretation

The broken code does not produce a compiler error or a runtime panic -- it *hangs*:

```
Output: 1
Output: 1
Output: 1
Output: 1
... (continues printing forever)
```

The program never terminates because:
1. The loop condition (`JmpNz` with nonzero top) is always true
2. There is no fuel limit to stop the loop
3. The `max_output` field is never checked

This is a denial-of-service vulnerability. In a WASM runtime, a module that runs forever prevents other modules from executing and wastes host resources. Fuel metering is the primary defense against this class of bugs.
