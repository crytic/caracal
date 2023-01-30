Run detectors
```bash
cargo run --bin starknet-static-analysis -- examples\unused_return.cairo
```

Print CFG
```bash
cargo run --bin starknet-static-analysis -- examples\unused_return.cairo --print cfg
cargo run --bin starknet-static-analysis -- examples\unused_return.cairo --print cfg-optimized
```
