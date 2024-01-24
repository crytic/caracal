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
Precompiled binaries are available on our [releases page](https://github.com/crytic/caracal/releases). If you are using Cairo compiler 1.x.x uses the binary v0.1.x otherwise if you are using the Cairo compiler 2.x.x uses v0.2.x.

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
To use with a standalone cairo file and you have a local cairo compiler binary it's enough to point it to the file. Otherwise otherwise a bundled compiler is used and you need to pass the path to the [corelib](https://github.com/starkware-libs/cairo/tree/main/corelib) library either with the `--corelib` cli option or by setting the `CORELIB_PATH` environment variable.  
Run detectors:
```bash
caracal detect path/file/to/analyze
```
```bash
caracal detect path/file/to/analyze --corelib path/to/corelib/src
```
Run printers:
```bash
caracal print path/file/to/analyze --printer printer_to_use --corelib path/to/corelib/src
```
### Cairo project
If you have a cairo project with multiple files and contracts you may need to specify which contracts with `--contract-path`. The local cairo compiler binary is used if available otherwise a bundled compiler is used. In the latter case you also need to specify the corelib as explained above for the standalone case. The path is the directory where `cairo_project.toml` resides.  
Run detectors:
```bash
caracal detect path/to/dir
```
```bash
caracal detect path/to/dir --contract-path token::myerc20::... token::myerc721::...
```
Run printers:
```bash
caracal print path/to/dir --printer printer_to_use
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
Num | Detector | What it Detects | Impact | Confidence | Cairo
--- | --- | --- | --- | --- | ---
1 | `controlled-library-call` | Library calls with a user controlled class hash | High | Medium | 1 & 2
2 | `unchecked-l1-handler-from` | Detect L1 handlers without from address check | High | Medium | 1 & 2
3 | `felt252-unsafe-arithmetic` | Detect user controlled operations with felt252 type, which is not overflow/underflow safe | Medium | Medium | 1 & 2
4 | `reentrancy` | Detect when a storage variable is read before an external call and written after | Medium | Medium | 1 & 2
5 | `read-only-reentrancy` | Detect when a view function read a storage variable written after an external call | Medium | Medium | 1 & 2
6 | `unused-events` | Events defined but not emitted | Medium | Medium | 1 & 2
7 | `unused-return` | Unused return values | Medium | Medium | 1 & 2
8 | `unenforced-view` | Function has view decorator but modifies state | Medium | Medium | 1
9 | `tx-origin` | Detect usage of the transaction origin address as access control | Medium | Medium | 2
10 | `unused-arguments` | Unused arguments | Low | Medium | 1 & 2
11 | `reentrancy-benign` | Detect when a storage variable is written after an external call but not read before | Low | Medium | 1 & 2
12 | `reentrancy-events` | Detect when an event is emitted after an external call leading to out-of-order events | Low | Medium | 1 & 2
13 | `dead-code` | Private functions never used | Low | Medium | 1 & 2
14 | `use-after-pop-front` | Detect use of an array or a span after removing element(s) | Low | Medium | 1 & 2

The Cairo column represent the compiler version(s) for which the detector is valid.

## Printers
- `cfg`: Export the CFG of each function to a .dot file
- `callgraph`: Export function call graph to a .dot file

## How to contribute
Check the wiki on the following topics:
  * [How to write a detector](https://github.com/crytic/caracal/wiki/How-to-write-a-detector)
  * [How to write a printer](https://github.com/crytic/caracal/wiki/How-to-write-a-printer)

## Limitations
- Inlined functions are not handled correctly.
- Since it's working over the SIERRA representation it's not possible to report where an error is in the source code but we can only report SIERRA instructions/what's available in a SIERRA program.
