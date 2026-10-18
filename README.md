# mlua-decimal

An exact base-10 `Decimal` for [mlua](https://crates.io/crates/mlua): a userdata and the global `D`, on top of
[`rust_decimal`](https://crates.io/crates/rust_decimal) (28 digits; the scale is kept: `1.10 * 3` is `3.30`).

```rust
let lua = mlua::Lua::new();
mlua_decimal::install(&lua)?;
lua.load(r#"print(D"0.1" + D"0.2")"#).exec()?;   // 0.3
```

In Lua: `D"12.50"`, `D(3)`, `D.max(...)`, `D.min(...)`, `D.sum(list)`; arithmetic `+ - * / // %`; ordering `< <= ==`
between decimals; `d:round(digits, mode)` (`half_even` default, `half_up`, `half_down`, `down`, `up`, `floor`, `ceil`;
negative digits round to tens), `floor`, `ceil`, `trunc`, `abs`, `sign`, `scale`, `normalize`, `is_integer`,
`cmp`/`eq`/`lt`/`le`, `to_number`.

Deliberate rules, so a precision bug is loud:

* an integer may stand in for an operand, a Lua number **with a fraction** may not (`price * 0.1` is an error; write
  `price * D"0.1"`);
* Lua never asks a userdata about a number, so `d < 5` is an error and `d == 5` is `false`; use `d:lt(5)`, `d:eq(5)` or
  `d < D(5)`.

For Luau, `mlua_decimal::LUAU_TYPES` is a `declare` file for `Decimal` and `D`, so `--!strict` reports `d == 5` before
the script runs.

From Rust: `Dec(Decimal)` converts into Lua values, `as_decimal(&AnyUserData)` reads one back, and
`from_f64(f64)` gives a float's shortest round-trip digits as a `Decimal`.

Documentation: <https://vivainio.github.io/mlua-decimal/>

Features: `luau` (default), `lua54`, `lua51`, `luajit`; each also enables mlua's `vendored`.
