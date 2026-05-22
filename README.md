# Concurrent Register Simulator

A real-time TUI simulator designed to visualize and evaluate the consistency models of various shared memory registers in concurrent systems. This tool demonstrates how data propagates across different isolation and atomicity levels under multi-threaded read/write contention.

The primary objective of this project is to implement and visualize the concurrent register hierarchy outlined in Maurice Herlihy and Nir Shavit’s foundational textbook, *The Art of Multiprocessor Programming*. By systematically building up from single-reader, single-writer (SRSW) safe registers all the way to multi-reader, multi-writer (MRMW) atomic registers, the simulator provides a practical demonstration of how highly consistent, linearizable shared memory can be engineered from weaker base primitives. The underlying execution flows and matrix-handshake mechanics strictly follow the formal constructions detailed in their classic work, translating theoretical multiprocessor concepts into a concrete, interactive execution runtime.

---

## Supported Registers

The simulator models a progression of classic distributed shared-memory register types:

* **Safe binary SRSW registers** These are simulated with a random period of instability on `write`. These are the only ones not implemented from previous primitives.
* **Safe Registers:** MRSW variants where reads concurrent with a write can return any valid member of the data type.
* **Regular Registers:** SRSW and M-Valued MRSW variations where a read concurrent with a write returns either the old value or the new value.
* **Atomic Registers:** SRSW, MRSW, and multi-writer (MRMW) variants that guarantee linearizability using sequential timestamps and read/write matrix handshakes.

---

## How to Run and Use the Simulator

### Prerequisites

Make sure you have the Rust toolchain (compiler and Cargo package manager) installed on your system.

### Running the Application

To build and launch the TUI dashboard in release mode for optimal performance, run the following command in your terminal:

```bash
cargo run --release

```

### Navigating the TUI

1. **Configure Simulation Settings:** Use your arrow keys or configured UI controls to select the **Register Type**, adjust the **Number of Readers**, set the **Number of Reads**, and dial in individual thread pacing via **Writer/Reader Delays (ms)**.
2. **Select a Sequence:** Pick a preset sequence (Boolean or Numeric) from the configuration panels to feed data into the writing registers.
3. **Control Execution:** * Press the designated action keys to **Start** or **Reset** a simulation run.
* Use the **Pause/Resume** flag toggle to freeze the execution thread timelines mid-run to inspect data propagation states.


4. **Analyze Outputs:** Monitor the live writer updates pane alongside individual reader logs to observe how values step up or cross-propagate based on the chosen register's constraints.

---

## How to Run Tests

The repository includes deterministic unit tests to verify the step-by-step algorithms, timestamp escalation mechanics, and memory ownership constraints of the register matrices without thread flakiness.

To execute the entire test suite, run:

```bash
cargo test

```
