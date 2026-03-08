---
id: fuel-metering
phase: 24
phase_title: "Advanced WASM Systems Patterns"
sequence: 2
title: Fuel Metering — Preventing Infinite Loops
hints:
  - WASM runtimes use "fuel" to limit how many instructions a module can execute
  - Each instruction should consume fuel before executing
  - Without fuel metering, an infinite loop runs forever
  - Use checked_sub to safely decrement fuel and detect exhaustion
---

## Description

WASM runtimes like Wasmtime support "fuel metering": the host assigns a fuel budget before calling into a module. Each WASM instruction consumes some fuel. When the fuel runs out, execution halts with a deterministic error instead of running forever. This prevents denial-of-service from malicious or buggy modules. This kata builds a simple virtual machine with fuel metering and shows what happens when fuel is not checked.

## Broken Code

```rust
#[derive(Debug, Clone)]
enum Op {
    Push(i64),
    Add,
    Dup,    // Duplicate top of stack
    JmpNz(usize), // Jump to instruction index if top != 0
    Print,
    Halt,
}

struct Vm {
    stack: Vec<i64>,
    fuel: u64,
    output: Vec<i64>,
}

impl Vm {
    fn new(fuel: u64) -> Self {
        Vm {
            stack: Vec::new(),
            fuel,
            output: Vec::new(),
        }
    }

    fn run(&mut self, program: &[Op]) -> Vec<i64> {
        let mut pc = 0;

        while pc < program.len() {
            // BUG: Fuel is never consumed!
            // An infinite loop runs forever without decrementing fuel.
            match &program[pc] {
                Op::Push(val) => self.stack.push(*val),
                Op::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Op::Dup => {
                    let top = *self.stack.last().unwrap();
                    self.stack.push(top);
                }
                Op::JmpNz(target) => {
                    let top = self.stack.pop().unwrap();
                    if top != 0 {
                        pc = *target;
                        continue;
                    }
                }
                Op::Print => {
                    let val = *self.stack.last().unwrap();
                    self.output.push(val);
                }
                Op::Halt => break,
            }
            pc += 1;
        }

        self.output.clone()
    }
}

fn main() {
    let mut vm = Vm::new(100);

    // A program that loops: pushes 1, duplicates, jumps back if nonzero
    // Without fuel, this runs forever!
    let program = vec![
        Op::Push(1),     // 0: push 1
        Op::Dup,         // 1: duplicate top
        Op::Print,       // 2: print top
        Op::JmpNz(1),   // 3: if top != 0, jump to 1 (infinite loop!)
        Op::Halt,        // 4: never reached
    ];

    let output = vm.run(&program);
    println!("Output length: {}", output.len());

    // Should have stopped after fuel ran out
    assert!(
        output.len() <= 100,
        "Ran {} iterations -- fuel metering is broken!",
        output.len()
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
    JmpNz(usize),
    Print,
    Halt,
}

#[derive(Debug)]
enum VmError {
    OutOfFuel { executed: u64 },
    StackUnderflow,
}

struct Vm {
    stack: Vec<i64>,
    fuel: u64,
    instructions_executed: u64,
    output: Vec<i64>,
}

impl Vm {
    fn new(fuel: u64) -> Self {
        Vm {
            stack: Vec::new(),
            fuel,
            instructions_executed: 0,
            output: Vec::new(),
        }
    }

    fn consume_fuel(&mut self, cost: u64) -> Result<(), VmError> {
        match self.fuel.checked_sub(cost) {
            Some(remaining) => {
                self.fuel = remaining;
                self.instructions_executed += 1;
                Ok(())
            }
            None => Err(VmError::OutOfFuel {
                executed: self.instructions_executed,
            }),
        }
    }

    fn run(&mut self, program: &[Op]) -> Result<Vec<i64>, VmError> {
        let mut pc = 0;

        while pc < program.len() {
            // Correct: consume fuel BEFORE executing each instruction
            self.consume_fuel(1)?;

            match &program[pc] {
                Op::Push(val) => self.stack.push(*val),
                Op::Add => {
                    if self.stack.len() < 2 {
                        return Err(VmError::StackUnderflow);
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Op::Dup => {
                    let top = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                    self.stack.push(top);
                }
                Op::JmpNz(target) => {
                    let top = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    if top != 0 {
                        pc = *target;
                        continue;
                    }
                }
                Op::Print => {
                    let val = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                    self.output.push(val);
                }
                Op::Halt => break,
            }
            pc += 1;
        }

        Ok(self.output.clone())
    }
}

fn main() {
    let mut vm = Vm::new(100);

    let program = vec![
        Op::Push(1),
        Op::Dup,
        Op::Print,
        Op::JmpNz(1),
        Op::Halt,
    ];

    match vm.run(&program) {
        Ok(output) => println!("Completed with {} outputs", output.len()),
        Err(VmError::OutOfFuel { executed }) => {
            println!("Halted: out of fuel after {} instructions", executed);
            println!("Output collected: {} values", vm.output.len());
        }
        Err(e) => println!("Error: {:?}", e),
    }

    assert!(
        vm.output.len() <= 100,
        "Should stop within fuel budget, got {} outputs",
        vm.output.len()
    );

    println!("Fuel metering working correctly!");
}
```

## Explanation

The broken version has a `fuel` field but never decrements it. The infinite loop (`JmpNz(1)` with a nonzero stack top) runs without limit, accumulating output until memory is exhausted or the program is killed. The assertion at the end is never reached because the loop never terminates.

**How fuel metering works in Wasmtime:**

1. The host calls `store.set_fuel(1_000_000)` before executing guest code
2. Wasmtime injects fuel checks at the start of each basic block (a sequence of instructions without branches)
3. When the fuel budget reaches zero, Wasmtime raises an `OutOfFuel` trap
4. The host catches the trap and can decide what to do: kill the module, allocate more fuel, or return a partial result

**Why fuel is checked before execution:**

If fuel were checked after an instruction, the last instruction could have arbitrary side effects before the check triggers. Checking before ensures the instruction only runs if there is fuel available. This is the same principle as the sandbox allocation check (Phase 23, Kata 2): verify the budget before committing the action.

**Fuel costs per instruction:**

In real runtimes, different instructions have different fuel costs:
- `i32.add`: 1 fuel unit
- `memory.load`: 2 fuel units (memory access is more expensive)
- `call`: 5 fuel units (function calls are expensive)
- `memory.grow`: 1000 fuel units (page allocation is very expensive)

Our simplified VM uses 1 fuel per instruction, but the principle is the same.

**Use cases for fuel metering:**
- **Serverless functions** (Cloudflare Workers, Fastly Compute): limit CPU time per request
- **Plugin systems**: prevent a buggy plugin from hanging the host
- **Smart contracts**: gas metering ensures bounded execution cost
- **Interactive sandboxes**: let users run untrusted code with safe limits

The invariant violated in the broken code: **every instruction must consume fuel before executing; without fuel checks, untrusted code can run indefinitely.**

## ⚠️ Caution

- Fuel costs must be calibrated for the workload. A loop that runs 10 billion iterations needs more fuel than one that runs 10. Uniform fuel costs per instruction may not reflect actual execution time.
- Without fuel metering, untrusted WASM code can run forever (infinite loop), consuming CPU without limit.

## 💡 Tips

- Assign higher fuel costs to expensive operations (memory allocation, function calls) and lower costs to simple arithmetic.
- Wasmtime's fuel metering is the production standard — use it for real WASM hosts.
- Fuel is consumed before execution, so the module traps cleanly rather than consuming resources it should not.

## Compiler Error Interpretation

The broken code does not produce a compiler error -- it compiles and runs. The problem is that it runs *forever* (or until killed):

```
Output length: [program hangs here, never prints]
```

Since the loop never terminates, `vm.run()` never returns. The program appears to hang. If you interrupt it, you would see that `output.len()` is millions or billions -- the program was accumulating values without bound.

In the correct version, after 100 fuel units:
```
Halted: out of fuel after 100 instructions
Output collected: 33 values
```

The loop executed exactly 100 instructions (each iteration is 3 instructions: Dup, Print, JmpNz), producing 33 printed values before halting cleanly with an error.
