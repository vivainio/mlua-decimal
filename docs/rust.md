# Using it from Rust

| | |
| --- | --- |
| `install(&lua)` | defines the global `D` and registers the userdata |
| `Dec(Decimal)` | the userdata; converts into a Lua value, so it can be returned from a function or `set` as a global |
| `as_decimal(&AnyUserData)` | the `Decimal` inside a userdata, or `None` if it is another kind of userdata |
| `construct(&Value)` | what `D(x)` does: a string, an integer or a decimal, as a `Dec` |
| `from_f64(f64)` | a float as the decimal with its shortest round-trip digits (`0.1` is exactly `0.1`); `None` for NaN and infinity |
| `DECIMAL_RANGE` | the error text for a result beyond 28 digits |
| `LUAU_TYPES` | `declare` lines for the Luau type checker, see [Type checking](type-checking.md) |

## Passing decimals in and out

```rust
use mlua::{Lua, Value};
use mlua_decimal::{as_decimal, Dec};

let lua = Lua::new();
mlua_decimal::install(&lua)?;

// into a script
lua.globals().set("price", Dec("12.50".parse()?))?;

// a function that returns one
let tax = lua.create_function(|_, net: Value| {
    let Value::UserData(u) = net else { return Err(mlua::Error::runtime("expected a Decimal")) };
    let net = as_decimal(&u).ok_or_else(|| mlua::Error::runtime("expected a Decimal"))?;
    Ok(Dec(net * rust_decimal::Decimal::new(24, 2)))
})?;
lua.globals().set("tax", tax)?;

let s: String = lua.load("return tostring(tax(price))").eval()?;
assert_eq!(s, "3.0000");   // 12.50 * 0.24: the scales add up
```

## Floats from Lua

A Lua number reaches Rust as an `f64`. If you accept one where a decimal is wanted, convert it with `from_f64`, so
that `0.1` becomes `0.1` rather than `0.1000000000000000055511151231`. The crate itself refuses floats with a fraction
as operands; what your own functions accept is up to you.

## Errors

The errors are ordinary `mlua::Error::runtime` values with these messages:

* `Attempted to divide by zero.`
* `Value was either too large or too small for a Decimal.`
* `Cannot convert "abc" to a Decimal`
* `A Luau number with a fraction is not a Decimal operand (0.1): write D"0.1" for an exact value`
* `Unknown rounding mode "..." (half_even, half_up, half_down, down, up, floor, ceil)`
