# Steel6502

Steel6502 is a Rust command-line emulator for the MOS 6502 family,
currently centered around the W65C02S CPU model. It loads a ROM
image into an emulated 64KB address space (with a simple 32KB RAM + 32KB
ROM split), resets the CPU using the reset vector, executes instructions
until it encounters `BRK`, and then writes a dump of RAM to disk.

## What this project is

This repository implements a practical CPU + bus + memory emulator core
packaged as a CLI program:

-   **CPU core**: A `W65C02S` implementation with registers, status
    flags, reset/interrupt vectors, instruction decoding/execution, and
    structured error handling for invalid opcodes and operands.
-   **Bus + machine model**: A `Bus` trait and a `Machine`
    implementation that maps pages to RAM/ROM and enforces ROM read-only
    behavior.
-   **Memory system**: Page-based memory (256 bytes per page) and
    RAM/ROM segments built from pages, with helpers for loading images
    and dumping contents.
-   **CLI runner**: A simple executable that reads an input file, runs
    the emulator, and writes an output RAM image.

## Skills demonstrated

This project demonstrates:

-   **Systems programming in Rust**
    -   Trait-based hardware abstraction
    -   Ownership and borrowing in a stateful emulator loop
    -   Clean separation of CPU, bus, and memory layers
-   **CPU architecture and emulation design**
    -   Instruction fetch/decode/execute loop
    -   Reset behavior via interrupt vectors
    -   Explicit illegal opcode handling
    -   Structured execution termination via `BRK`
-   **Memory modeling**
    -   Page-based memory organization
    -   Explicit RAM/ROM segmentation
    -   Raw, performance-oriented memory access paths
-   **Command-line tooling**
    -   Argument parsing
    -   File I/O
    -   Deterministic output artifacts (RAM dump)

## Memory Map

Steel6502 emulates a simple 64KB address space:

-   `$0000–$7FFF` → 32KB RAM\
-   `$8000–$FFFF` → 32KB ROM

ROM is read-only. Writes to ROM are prevented at the bus layer.

## Input Format Expectations

Steel6502 loads the **upper 32KB** of the provided file (offset
`0x8000`) into the emulated ROM region mapped at `$8000–$FFFF`.

For best compatibility, provide a full **64KB memory image**, where:

-   `0x0000–0x7FFF` → RAM region (ignored on load)
-   `0x8000–0xFFFF` → ROM data to execute

The reset vector must be correctly configured in the ROM image for
proper execution.

## How to Use

### Prerequisites

-   Rust toolchain (Cargo)

### Build

``` bash
git clone https://github.com/JForte05/Steel6502.git
cd Steel6502
cargo build --release
```

### Run

``` bash
cargo run --release -- path/to/image.bin
```

You may optionally specify an output directory:

``` bash
cargo run --release -- -o path/to/output_dir path/to/image.bin
```

If `-o` is not provided, output files are written to the current working
directory.

## Execution Behavior

-   The CPU resets using the reset vector in ROM.
-   Instructions execute in a loop.
-   Execution halts when `BRK` is encountered.
-   After termination, RAM is dumped to disk.

## Output

After execution completes, Steel6502 writes a RAM dump file:

    <input_file_stem>_ram.bin

This file contains the full 32KB RAM contents after program execution.
