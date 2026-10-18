# Decimals in Lua

## Making one

```lua
local price = D"12.50"     -- from text; scientific notation works too: D"1.5e3"
local qty   = D(3)         -- an integer is exact
local same  = D(price)     -- a decimal stays as it is
```

`D(0.5)` is an error: a float with a fraction is not what its source text said. Write `D"0.5"`.

## Arithmetic

`+ - * / // %` and unary `-` work between two decimals, and an integer may stand in for either operand
(`price * 3`, `1 + price`). `//` is floor division (not on Lua 5.1 and LuaJIT, which have no such operator). The scale follows the operands, as in .NET's `decimal`:

```lua
print(D"1.10" * 3)      -- 3.30
print(D"10" / D"4")     -- 2.50
print(D"7" // 2)        -- 3
print(D"7.5" % 2)       -- 1.5
```

Division by zero is `Attempted to divide by zero.`, and a result beyond 28 digits is
`Value was either too large or too small for a Decimal.`

## Comparison

`<`, `<=` and `==` work between two decimals, ignoring the scale. `==` never calls a metamethod for a userdata against a
number, and `<` is an error, so a decimal and a Lua number do not compare with operators:

| | |
| --- | --- |
| `price == D"12.5"` | `true` |
| `price == 12.5` | **`false`, always** |
| `price < 20` | an error |
| `price < D(20)` | `true` |

The `false, always` row is the dangerous one, since nothing fails and the script takes the wrong branch. Use the
[type checker](type-checking.md) to turn it into an error, or compare through the methods:

```lua
price:lt(20)         -- true; the other side is a decimal or an integer
price:le(20)
price:eq(D"12.5")    -- a fraction needs D"..."
price:cmp(20)        -- -1, 0 or 1
```

## Methods

| | |
| --- | --- |
| `d:round([digits [, mode]])` | `digits` may be negative; modes: `half_even` (default), `half_up`, `half_down`, `down`, `up`, `floor`, `ceil` |
| `d:floor()`, `d:ceil()`, `d:trunc()`, `d:abs()`, `d:sign()` | |
| `d:scale()` | the digits after the point: `D"1.50":scale()` is 2 |
| `d:normalize()` | without trailing zeros: `D"1.50"` becomes `1.5` |
| `d:is_integer()` | |
| `d:cmp(x)`, `d:eq(x)`, `d:lt(x)`, `d:le(x)` | `x` is a decimal or an integer |
| `d:to_number()` | the explicit, lossy way out to a Lua number |
| `tostring(d)` | the digits, with their scale |
| `D.max(...)`, `D.min(...)`, `D.sum(list)` | over decimals and integers |

```lua
print(D"2.5":round())                -- 2   (half to even)
print(D"2.5":round(0, "half_up"))    -- 3
print(D"2.345":round(2))             -- 2.34
print(D"1234":round(-2))             -- 1200
```

## What a decimal is not

A decimal is not a number to Lua's own functions: `math.floor(d)`, `string.format("%d", d)` and `for i = 1, d` all
fail, and a decimal used as a table key matches only itself, not an equal decimal. Use `d:floor()`, `tostring(d)` or
`d:to_number()` for those.
