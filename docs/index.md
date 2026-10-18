# mlua-decimal

An exact base-10 `Decimal` for [mlua](https://crates.io/crates/mlua): a userdata and the global `D`, on top of
[`rust_decimal`](https://crates.io/crates/rust_decimal). Up to 28 digits, so `0.1 + 0.2` is `0.3` and a price times a
quantity has no rounding noise.

```rust
let lua = mlua::Lua::new();
mlua_decimal::install(&lua)?;
lua.load(r#"print(D"0.1" + D"0.2")"#).exec()?;   // 0.3
```

* **Exact.** Arithmetic is decimal; the scale is kept (`1.10 * 3` is `3.30`, `100.00 + 24` is `124.00`) and equality
  ignores it (`1.0 == 1.00`).
* **Loud.** A Lua number with a fraction is never an operand (`price * 0.1` is an error; write `price * D"0.1"`), and
  there is no NaN or infinity. A silent precision bug becomes a failure.
* **Typed.** For Luau, [`LUAU_TYPES`](type-checking.md) declares `Decimal` and `D`, so `--!strict` reports `price == 12.5`
  before the script runs. At run time that comparison is just `false`.
* **Small.** The crate depends on `mlua` and `rust_decimal` only.

## Install

```toml
[dependencies]
mlua-decimal = { git = "https://github.com/vivainio/mlua-decimal" }
```

The Lua flavour is a feature: `luau` (the default), `lua54`, `lua51` or `luajit`. Each also enables mlua's `vendored`.
If your application already depends on `mlua` with its own features, use `default-features = false` and enable the
flavour you use, so the two agree.

```toml
mlua-decimal = { git = "https://github.com/vivainio/mlua-decimal", default-features = false, features = ["lua54"] }
```

Next: [using it from Rust](rust.md), [decimals in Lua](lua.md), [type checking](type-checking.md).
