Embedded or dynamic linking of an object test.

Run embedded:
```bash
cargo run --example hello_world
```

Run dynamically loaded:
```bash
cargo build -p dyn-plugin-test | cargo run --example hello_world --features="dynamic"
```
