//! An exact base-10 `Decimal` for [mlua](https://docs.rs/mlua): a userdata plus the global `D`.
//!
//! ```
//! let lua = mlua::Lua::new();
//! mlua_decimal::install(&lua).unwrap();
//! let s: String = lua.load(r#"return tostring(D"0.1" + D"0.2")"#).eval().unwrap();
//! assert_eq!(s, "0.3");
//! ```
//!
//! Arithmetic (`+ - * / // %`, unary `-`) and ordering (`< <= ==`) work between two decimals. An integer may stand
//! in for either operand; any other Lua number may not, because a float such as `0.1` is not what its source text
//! said (`d * 0.1` is an error: write `d * D"0.1"`). Lua never calls an ordering or equality metamethod for a
//! userdata against a number, so `d < 5` is an error and `d == 5` is false: use `d:lt(5)`, `d:eq(5)`, `d:cmp(5)`
//! or `D(5)`.
//!
//! [`LUAU_TYPES`] holds `declare` lines for the Luau type checker, so `--!strict` scripts reject `d == 5`.

use std::cmp::Ordering;

use mlua::{
    AnyUserData, Lua, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, UserDataRef,
    Value as LV,
};
use rust_decimal::prelude::*;
use rust_decimal::{Decimal, RoundingStrategy};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dec(pub Decimal);

fn rt(msg: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(msg.into())
}

/// A Luau number as a decimal: integers are exact, a number with a fraction is refused.
fn from_number(f: f64) -> mlua::Result<Decimal> {
    if f.is_finite() && f.fract() == 0.0 {
        if let Some(d) = Decimal::from_f64(f) {
            return Ok(d);
        }
    }
    Err(rt(format!(
        "A Luau number with a fraction is not a Decimal operand ({f}): write D\"{f}\" for an exact value"
    )))
}

/// A decimal, integer or (for constructors) string.
fn operand(v: &LV) -> mlua::Result<Decimal> {
    match v {
        LV::UserData(u) => u
            .borrow::<Dec>()
            .map(|d| d.0)
            .map_err(|_| rt("Expected a Decimal or an integer")),
        LV::Integer(i) => Ok(Decimal::from(*i)),
        LV::Number(f) => from_number(*f),
        other => Err(rt(format!(
            "Expected a Decimal or an integer, got {}",
            other.type_name()
        ))),
    }
}

/// `D(x)`: a decimal from its text, an integer or a decimal.
pub fn construct(v: &LV) -> mlua::Result<Dec> {
    match v {
        LV::String(s) => {
            let text = s.to_string_lossy();
            let t = text.trim();
            t.parse::<Decimal>()
                .or_else(|_| Decimal::from_scientific(t))
                .map(Dec)
                .map_err(|_| rt(format!("Cannot convert \"{text}\" to a Decimal")))
        }
        other => operand(other).map(Dec),
    }
}

fn checked(r: Option<Decimal>) -> mlua::Result<Dec> {
    r.map(Dec).ok_or_else(|| rt(DECIMAL_RANGE))
}

fn nonzero(d: Decimal) -> mlua::Result<Decimal> {
    if d.is_zero() {
        Err(rt("Attempted to divide by zero."))
    } else {
        Ok(d)
    }
}

fn strategy(name: Option<&str>) -> mlua::Result<RoundingStrategy> {
    Ok(match name.unwrap_or("half_even") {
        "half_even" => RoundingStrategy::MidpointNearestEven,
        "half_up" => RoundingStrategy::MidpointAwayFromZero,
        "half_down" => RoundingStrategy::MidpointTowardZero,
        "down" => RoundingStrategy::ToZero,
        "up" => RoundingStrategy::AwayFromZero,
        "floor" => RoundingStrategy::ToNegativeInfinity,
        "ceil" => RoundingStrategy::ToPositiveInfinity,
        other => {
            return Err(rt(format!(
                "Unknown rounding mode \"{other}\" (half_even, half_up, half_down, down, up, floor, ceil)"
            )))
        }
    })
}

fn round(d: Decimal, digits: i64, mode: RoundingStrategy) -> mlua::Result<Dec> {
    if digits >= 0 {
        let digits = u32::try_from(digits).map_err(|_| rt(DECIMAL_RANGE))?;
        return Ok(Dec(d.round_dp_with_strategy(digits, mode)));
    }
    // a negative number of digits rounds to tens, hundreds, ...
    let unit = (0..-digits)
        .try_fold(Decimal::ONE, |unit, _| unit.checked_mul(Decimal::TEN))
        .ok_or_else(|| rt(DECIMAL_RANGE))?;
    checked((d / unit).round_dp_with_strategy(0, mode).checked_mul(unit))
}

fn binary<M: UserDataMethods<Dec>>(
    m: &mut M,
    meta: MetaMethod,
    f: fn(Decimal, Decimal) -> mlua::Result<Dec>,
) {
    m.add_meta_function(meta, move |_, (a, b): (LV, LV)| {
        f(operand(&a)?, operand(&b)?)
    });
}

impl UserData for Dec {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        // `typeof(d)`
        f.add_meta_field("__type", "Decimal");
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        binary(m, MetaMethod::Add, |a, b| checked(a.checked_add(b)));
        binary(m, MetaMethod::Sub, |a, b| checked(a.checked_sub(b)));
        binary(m, MetaMethod::Mul, |a, b| checked(a.checked_mul(b)));
        binary(m, MetaMethod::Div, |a, b| {
            checked(a.checked_div(nonzero(b)?))
        });
        binary(m, MetaMethod::Mod, |a, b| {
            checked(a.checked_rem(nonzero(b)?))
        });
        binary(m, MetaMethod::IDiv, |a, b| {
            checked(a.checked_div(nonzero(b)?).map(|q| q.floor()))
        });
        m.add_meta_method(MetaMethod::Unm, |_, a, ()| Ok(Dec(-a.0)));
        m.add_meta_method(MetaMethod::ToString, |_, a, ()| Ok(a.0.to_string()));
        // Luau only calls these with a userdata on both sides
        m.add_meta_function(
            MetaMethod::Eq,
            |_, (a, b): (UserDataRef<Dec>, UserDataRef<Dec>)| Ok(a.0 == b.0),
        );
        m.add_meta_function(
            MetaMethod::Lt,
            |_, (a, b): (UserDataRef<Dec>, UserDataRef<Dec>)| Ok(a.0 < b.0),
        );
        m.add_meta_function(
            MetaMethod::Le,
            |_, (a, b): (UserDataRef<Dec>, UserDataRef<Dec>)| Ok(a.0 <= b.0),
        );

        // comparison with an integer or a decimal
        m.add_method("cmp", |_, a, b: LV| {
            Ok(match a.0.cmp(&operand(&b)?) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            })
        });
        m.add_method("eq", |_, a, b: LV| Ok(a.0 == operand(&b)?));
        m.add_method("lt", |_, a, b: LV| Ok(a.0 < operand(&b)?));
        m.add_method("le", |_, a, b: LV| Ok(a.0 <= operand(&b)?));

        // d:round(digits [, mode]): `digits` may be negative; the default mode is half_even
        m.add_method(
            "round",
            |_, a, (digits, mode): (Option<i64>, Option<String>)| {
                round(a.0, digits.unwrap_or(0), strategy(mode.as_deref())?)
            },
        );
        m.add_method("floor", |_, a, ()| Ok(Dec(a.0.floor())));
        m.add_method("ceil", |_, a, ()| Ok(Dec(a.0.ceil())));
        m.add_method("trunc", |_, a, ()| Ok(Dec(a.0.trunc())));
        m.add_method("abs", |_, a, ()| Ok(Dec(a.0.abs())));
        m.add_method("sign", |_, a, ()| {
            Ok(match a.0.cmp(&Decimal::ZERO) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            })
        });
        // the number of digits after the decimal point: D"1.50":scale() is 2
        m.add_method("scale", |_, a, ()| Ok(a.0.scale()));
        m.add_method("normalize", |_, a, ()| Ok(Dec(a.0.normalize())));
        m.add_method("is_integer", |_, a, ()| Ok(a.0.is_integer()));
        // the explicit, lossy way out
        m.add_method("to_number", |_, a, ()| Ok(a.0.to_f64().unwrap_or(f64::NAN)));
    }
}

fn extreme(args: mlua::Variadic<LV>, name: &str, keep: Ordering) -> mlua::Result<Dec> {
    let mut best: Option<Decimal> = None;
    for a in args.iter() {
        let d = operand(a)?;
        best = Some(match best {
            Some(b) if d.cmp(&b) != keep => b,
            _ => d,
        });
    }
    best.map(Dec)
        .ok_or_else(|| rt(format!("D.{name} needs at least one value")))
}

/// The global `D`: `D"1.50"`, `D(3)`, `D.max(a, b, ...)`, `D.min(...)`, `D.sum(t)`.
pub fn install(lua: &Lua) -> mlua::Result<()> {
    let d = lua.create_table()?;
    d.set(
        "max",
        lua.create_function(|_, args: mlua::Variadic<LV>| extreme(args, "max", Ordering::Greater))?,
    )?;
    d.set(
        "min",
        lua.create_function(|_, args: mlua::Variadic<LV>| extreme(args, "min", Ordering::Less))?,
    )?;
    d.set(
        "sum",
        lua.create_function(|_, items: Table| {
            let mut sum = Decimal::ZERO;
            for v in items.sequence_values::<LV>() {
                sum = sum
                    .checked_add(operand(&v?)?)
                    .ok_or_else(|| rt(DECIMAL_RANGE))?;
            }
            Ok(Dec(sum))
        })?,
    )?;
    let meta = lua.create_table()?;
    meta.set(
        "__call",
        lua.create_function(|_, (_, v): (LV, LV)| construct(&v))?,
    )?;
    d.set_metatable(Some(meta))?;
    lua.globals().set("D", d)
}

/// Whether a Lua value is a decimal, and which.
pub fn as_decimal(u: &AnyUserData) -> Option<Decimal> {
    u.borrow::<Dec>().ok().map(|d| d.0)
}

/// The error text for a result beyond 28 digits.
pub const DECIMAL_RANGE: &str = "Value was either too large or too small for a Decimal.";

/// Luau `declare` lines for `Decimal` and `D`, for `luau-analyze` style definition files.
pub const LUAU_TYPES: &str = include_str!("../decimal.d.luau");

/// A float as the decimal with its shortest round-trip digits (`0.1` is exactly 0.1). NaN and infinity have none.
pub fn from_f64(f: f64) -> Option<Decimal> {
    if !f.is_finite() {
        return None;
    }
    Decimal::from_scientific(&format!("{f:e}"))
        .ok()
        .or_else(|| Decimal::from_str(&format!("{f}")).ok())
}
