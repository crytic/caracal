# Caracal

Caracal is a static analyzer tool over the SIERRA representation for Starknet smart contracts.

## Features
- Detectors to detect vulnerable Cairo code
- Printers to report information
- Taint analysis
- Data flow analysis framework
- Easy to run in Scarb projects

## Installation

### Precompiled binaries
Precompiled binaries are available on our [releases page](https://github.com/crytic/caracal/releases).

### Building from source
You need the Rust compiler and Cargo.  
Building from git:
```bash
cargo install --git https://github.com/crytic/caracal --profile release --force
```
Building from a local copy:
```bash
git clone https://github.com/crytic/caracal
cd caracal
cargo install --path . --profile release --force
```

## Usage
List detectors:
```bash
caracal detectors
```
List printers:
```bash
caracal printers
```
### Standalone
To use with a standalone cairo file you need to pass the path to the [corelib](https://github.com/starkware-libs/cairo/tree/main/corelib) library either with the `--corelib` cli option or by setting the `CORELIB_PATH` environment variable.  
Run detectors:
```bash
caracal detect path/file/to/analyze --corelib path/to/corelib/src
```
Run printers:
```bash
caracal print path/file/to/analyze --printer printer_to_use --corelib path/to/corelib/src
```
### Scarb
If you have a project that uses Scarb you need to add the following in Scarb.toml:
```bash
[[target.starknet-contract]]
sierra = true

[cairo]
sierra-replace-ids = true
```
Then pass the path to the directory where Scarb.toml resides.  
Run detectors:
```bash
caracal detect path/to/dir
```
Run printers:
```bash
caracal print path/to/dir --printer printer_to_use
```

## Detectors
Num | Detector | What it Detects | Impact | Confidence
--- | --- | --- | --- | ---
1 | `controlled-library-call` | Library calls with a user controlled class hash | High | Medium
2 | `unchecked-l1-handler-from` | Detect L1 handlers without from address check | High | Medium
3 | `reentrancy` | Detect when a storage variable is read before an external call and written after | Medium | Medium
4 | `unused-events` | Events defined but not emitted | Medium | Medium
5 | `unused-return` | Unused return values | Medium | Medium
6 | `unenforced-view` | Function has view decorator but modifies state | Medium | Medium
7 | `unused-arguments` | Unused arguments | Low | High
8 | `reentrancy-benign` | Detect when a storage variable is written after an external call but not read before | Low | Medium
9 | `reentrancy-events` | Detect when an event is emitted after an external call leading to out-of-order events | Low | Medium
10 | `dead-code` | Private functions never used | Low | Medium

## Printers
- `cfg`: Export the CFG of each function in a .dot file
- `callgraph`: Export function call graph to a .dot file

## How to contribute
Check the wiki on the following topics:
  * [How to write a detector](https://github.com/crytic/caracal/wiki/How-to-write-a-detector)
  * [How to write a printer](https://github.com/crytic/caracal/wiki/How-to-write-a-printer)

## Limitations
- At the moment only Cairo 1 is supported (compiler version up to 1.1.1).
- Inlined functions are not handled correctly.
- Since it's working over the SIERRA representation it's not possible to report where an error is in the source code but we can only report SIERRA instructions/what's available in a SIERRA program.
