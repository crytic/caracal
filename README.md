# TBD

TBD is a static analyzer tool over the SIERRA representation for Starknet smart contracts.

## Features
- Detectors to detect vulnerable Cairo code
- Printers to report informations
- Taint analysis
- Data flow analysis framework

## Usage
You need to pass the path to the [corelib](https://github.com/starkware-libs/cairo/tree/main/corelib) library either with the `--corelib` cli option or by setting the `CORELIB_PATH` environment variable.  
List detectors:
```bash
cargo run --release --bin starknet-static-analysis detectors
```
Run detectors:
```bash
cargo run --release --bin starknet-static-analysis detect path/file/to/analyze --corelib path/to/corelib/src
```
List printers:
```bash
cargo run --release --bin starknet-static-analysis printers
```
Run detectors:
```bash
cargo run --release --bin starknet-static-analysis print path/file/to/analyze --what printer_to_use --corelib path/to/corelib/src
```

## Detectors

Num | Detector | What it Detects | Impact | Confidence
--- | --- | --- | --- | ---
1 | `controlled-library-call` | Library calls with a user controlled class hash | High | Medium
2 | `unused-events` | Events defined but not emitted | Medium | Medium
3 | `dead-code` | Private functions never used | Low | High
4 | `unused-arguments` | Unused arguments | Low | High
5 | `unused-return` | Unused return values | Medium | Medium

## Printers
- `cfg`: Export the CFG of each function in a .dot file
- `cfg-optimized`: Export the CFG optimized of each function in a .dot file. Note now it's the same as cfg because the SIERRA representation doesn't have the pattern that was optimized anymore.  

## How to contribute
Check the wiki on the following topics:
  * [How to write a detector](https://github.com/crytic/starknet-static-analysis/wiki/How-to-write-a-detector)
  * [How to write a printer](https://github.com/crytic/starknet-static-analysis/wiki/How-to-write-a-printer)

## Limitations
- Since it's working over the SIERRA representation it's not possible to report where an error is in the source code but we can only report SIERRA instructions/what's available in a SIERRA program.
- Works correctly only with Starknet contracts that have at least one `view` or `external` function.
