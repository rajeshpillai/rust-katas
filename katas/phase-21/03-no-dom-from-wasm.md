---
id: no-dom-from-wasm
phase: 21
phase_title: "WASM in the Browser"
sequence: 3
title: WASM Cannot Touch the DOM — Host Callbacks Required
hints:
  - WASM modules have no direct access to browser APIs (DOM, fetch, etc.)
  - All interaction with the outside world must go through imported host functions
  - If the module tries to call a method that does not exist on its struct, the compiler rejects it
  - Use a trait to define the host callback interface and inject it into the module
---

## Description

WASM modules in the browser cannot directly access the DOM, `fetch`, `localStorage`, or any other Web API. These APIs exist in the JavaScript realm, not in the WASM execution environment. For the module to update the UI, it must call an imported host function (provided by JavaScript). The host function then manipulates the DOM on behalf of the module. This kata demonstrates that a WASM module trying to call methods that were never provided by the host will fail at compilation -- the capability simply does not exist.

## Broken Code

```rust
struct WasmModule {
    title: String,
    items: Vec<String>,
}

impl WasmModule {
    fn new(title: &str) -> Self {
        WasmModule {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    fn render_ui(&self) {
        // BUG: Trying to call DOM-like methods that do not exist on this struct.
        // In WASM, the module has no access to document, window, or any browser API.
        // These methods are not defined anywhere.
        self.set_inner_html(&self.title);
        for item in &self.items {
            self.append_child(item);
        }
    }
}

fn main() {
    let mut module = WasmModule::new("Shopping List");
    module.add_item("Apples");
    module.add_item("Bread");
    module.render_ui();
}
```

## Correct Code

```rust
/// Host callbacks -- these represent JavaScript functions that the host
/// provides to the WASM module. The module can only interact with the
/// browser through these imported functions.
trait HostCallbacks {
    fn set_title(&self, text: &str);
    fn append_list_item(&self, text: &str);
    fn clear_list(&self);
}

/// A browser host implementation that prints what DOM operations
/// would be performed (simulating JS interop).
struct BrowserHost;

impl HostCallbacks for BrowserHost {
    fn set_title(&self, text: &str) {
        println!("[DOM] Set title: '{}'", text);
    }

    fn append_list_item(&self, text: &str) {
        println!("[DOM] Append <li>{}</li>", text);
    }

    fn clear_list(&self) {
        println!("[DOM] Clear list");
    }
}

/// A test host that captures operations for verification.
struct TestHost {
    operations: std::cell::RefCell<Vec<String>>,
}

impl TestHost {
    fn new() -> Self {
        TestHost {
            operations: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl HostCallbacks for TestHost {
    fn set_title(&self, text: &str) {
        self.operations.borrow_mut().push(format!("set_title:{}", text));
    }

    fn append_list_item(&self, text: &str) {
        self.operations.borrow_mut().push(format!("append:{}", text));
    }

    fn clear_list(&self) {
        self.operations.borrow_mut().push("clear".to_string());
    }
}

struct WasmModule<H: HostCallbacks> {
    host: H,
    title: String,
    items: Vec<String>,
}

impl<H: HostCallbacks> WasmModule<H> {
    fn new(title: &str, host: H) -> Self {
        WasmModule {
            host,
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    fn render_ui(&self) {
        // Correct: call host-provided functions for all DOM operations.
        // The module never touches the DOM directly.
        self.host.clear_list();
        self.host.set_title(&self.title);
        for item in &self.items {
            self.host.append_list_item(item);
        }
    }
}

fn main() {
    // Production: use browser host
    let mut module = WasmModule::new("Shopping List", BrowserHost);
    module.add_item("Apples");
    module.add_item("Bread");
    module.render_ui();

    println!("---");

    // Test: capture operations for verification
    let test_host = TestHost::new();
    let mut test_module = WasmModule::new("Test List", test_host);
    test_module.add_item("Item A");
    test_module.render_ui();

    let ops = test_module.host.operations.borrow();
    println!("Captured operations: {:?}", *ops);
    assert_eq!(ops.len(), 3); // clear, set_title, append
}
```

## Explanation

The broken version tries to call `self.set_inner_html()` and `self.append_child()` -- methods that do not exist on `WasmModule`. The compiler rejects this with "no method named `set_inner_html` found for struct `WasmModule`". The module is trying to perform operations that are not part of its capabilities.

**Why WASM cannot touch the DOM:**

The browser DOM is a JavaScript object graph managed by the JavaScript engine. WASM modules run in their own linear memory space with no access to JavaScript objects. A WASM module cannot:
- Call `document.getElementById()`
- Modify `element.innerHTML`
- Add event listeners
- Access `window.localStorage`

None of these APIs exist in the WASM address space.

**How the host callback pattern works:**

1. The host (JavaScript) defines callback functions and passes them as imports when instantiating the WASM module
2. The WASM module calls these imported functions when it needs to interact with the browser
3. The host function executes the DOM operation on behalf of the module

In `wasm-bindgen`, this is automated: you write `web_sys::document()` in Rust, and `wasm-bindgen` generates JavaScript glue code that calls `document` on your behalf. But under the hood, it is always a host callback.

**The testability benefit:**

The correct version can use `TestHost` to verify behavior without a browser. The `operations` vector captures every DOM operation the module would have performed. This makes testing fast, deterministic, and does not require a browser environment.

The invariant violated in the broken code: **WASM modules cannot access browser APIs directly; all external interaction must go through imported host functions.**

## ⚠️ Caution

- WASM has no access to browser APIs (DOM, fetch, localStorage) without host-provided imports. Direct access is impossible by design.
- `wasm-bindgen` and `web_sys` provide DOM access but add bridge overhead. Use them sparingly for performance-sensitive paths.

## 💡 Tips

- Return data from WASM and let JavaScript handle DOM updates. This keeps the WASM module portable.
- Use the "command pattern": WASM returns a list of DOM operations, JavaScript executes them in a batch.
- Test WASM modules with a mock host that records operations instead of executing them.

## Compiler Error Interpretation

```
error[E0599]: no method named `set_inner_html` found for reference
              `&WasmModule` in the current scope
  --> src/main.rs:20:14
   |
20 |         self.set_inner_html(&self.title);
   |              ^^^^^^^^^^^^^^ method not found in `&WasmModule`

error[E0599]: no method named `append_child` found for reference
              `&WasmModule` in the current scope
  --> src/main.rs:22:18
   |
22 |             self.append_child(item);
   |                  ^^^^^^^^^^^^ method not found in `&WasmModule`
```

The compiler errors are straightforward:

1. **"no method named `set_inner_html` found for reference `&WasmModule`"** -- the struct `WasmModule` does not have this method. It was never defined because the module is not a DOM element.
2. **"no method named `append_child` found"** -- same issue. The module is trying to perform an operation it has no capability for.

This directly models the WASM experience: if the host does not provide an import function, the module cannot call it. The absence is structural, not a permission check. The function does not exist in the module's world.

---

| [Prev: Batch Calls vs Chatty Calls — Minimizing Boundary Crossings](#/katas/batch-vs-chatty-calls) | [Next: Allocation Minimization — Reuse Buffers Across Calls](#/katas/allocation-minimization) |
