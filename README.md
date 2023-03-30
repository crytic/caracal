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

## How to write a detector
Add a test in [tests/detectors/](tests/detectors/) with the name of your detector and an example of what it should detect.  
Create your new detector in [detectors/](src/detectors/).  
Add your detector in [get_detectors](src/detectors/mod.rs).  
It needs to be a struct which implements the [Detector](src/detectors/detector.rs) trait. `name`/`description`/`impact`/`confidence` functions are self explaining.  
In the `run` function you will get a reference to the [CoreUnit](src/core/core_unit.rs) object, as of now you only need to get the compilation unit from it and then it's likely you need to decide to iterate over all the functions or only user defined (see [CompilationUnit](src/core/compilation_unit.rs)).  
Depending on the what your detector needs to do you can use metadata from the [Function](src/core/function.rs) object such as the events the current function emits, or iterate over the SIERRA statements.  
You must return a `Vec<Result>` so when you find something that should be reported add a [Result](src/detectors/detector.rs) element in your array that at the end you will return.  
Now that your detector is ready run `cargo test`, it will fail. We do snapshot testing for the detectors using the [insta](https://docs.rs/insta/latest/insta/) crate.  
To make `cargo test` not fail run `cargo insta review` (if you don't have it installed do `cargo install cargo-insta`).  
See the proposed output and if it matches what you expect accept it otherwise go back to your detector and improve it.  
Lastly run `cargo fmt`.

## How to write a printer
Read [how to write a detector](#how-to-write-a-detector).  
It's the same process except you create your printer in [printers/](src/printers/), implements the [Printer](src/printers/printer.rs) trait and add it in [get_printers](src/printers/mod.rs).  
Additionally in the `run` function you will get a [PrintOpts](src/printers/printer.rs) argument.  
At the moment printers don't have tests.

## Limitations
- Since it's working over the SIERRA representation it's not possible to report where an error is in the source code but we can only report SIERRA instructions/what's available in a SIERRA program.
- Works correctly only with Starknet contracts that have at least one `view` or `external` function.