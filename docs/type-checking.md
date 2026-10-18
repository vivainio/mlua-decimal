# Type checking

The Luau checker reads only Luau source and a definitions file; it knows nothing about a Rust userdata. The crate
ships the definitions as `mlua_decimal::LUAU_TYPES` (the file `decimal.d.luau` in the repository): a
`declare extern type Decimal` with every method and operator, and `declare D`.

## Using it

Write the definitions to a file and hand it to your checker. With
[`luau-lsp`](https://github.com/JohnnyMorganz/luau-lsp):

```rust
std::fs::write("decimal.d.luau", mlua_decimal::LUAU_TYPES)?;
```

```bash
luau-lsp analyze --definitions=decimal.d.luau script.luau
```

For an editor, list the file under `luau-lsp.types.definitionFiles`. If your application has definitions of its own,
concatenate: a definitions file is just a list of `declare` lines.

## What it catches

Start the script with `--!strict`.

```lua
--!strict
local price = D"12.50"
local a = price == 12.5      -- TypeError: Cannot compare unrelated types 'Decimal' and 'number'
local b = price < 20         -- TypeError: the same, with '<'
local c = price == D"12.5"   -- fine
local d = price < D(20)      -- fine
```

Without `--!strict` the checker reports nothing here.

## What it cannot

* `price * 0.1` type-checks, because arithmetic accepts `number` and Luau has no integer type to say "integers only".
  It is an error when it runs.
* The checker only knows a value is a `Decimal` if it came through something typed. A value your application hands
  to the script as `any` (a cell, a variable) is not checked until you wrap it: `D` takes `any` on purpose, so
  `local price = D(cell)` gives the checker a `Decimal` to work with.

## Keeping the definitions honest

The definitions are written by hand, next to the Rust methods. `tests/types.sh` checks them with `luau-lsp`: a sample
script must pass `--!strict`, and two scripts with `price == 12.5` and `price < 20` must fail. It runs in CI.
