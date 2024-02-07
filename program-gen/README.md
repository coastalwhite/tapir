

# Compiling staticlib

```
cargo rustc --lib --release --crate-type=staticlib
export RVIGEN_DIR="$PWD"

cd /path/to/verilator/project
verilator <USUAL FLAGS> -CFLAGS "-I$RVIGEN_DIR" -LDFLAGS "-L$RVIGEN_DIR/target/release -lrv_input_gen"
```