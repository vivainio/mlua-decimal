use mlua::Lua;

fn lua() -> Lua {
    let lua = Lua::new();
    mlua_decimal::install(&lua).unwrap();
    lua
}

fn eval(code: &str) -> String {
    lua().load(code).eval::<String>().unwrap()
}

fn fails(code: &str, needle: &str) {
    let err = lua().load(code).exec().unwrap_err().to_string();
    assert!(err.contains(needle), "{err}");
}

#[test]
fn arithmetic_is_exact_and_keeps_scale() {
    assert_eq!(eval(r#"return tostring(D"0.1" + D"0.2")"#), "0.3");
    assert_eq!(eval(r#"return tostring(D"1.10" * 3)"#), "3.30");
    assert_eq!(eval(r#"return tostring(D"10" / D"4")"#), "2.50");
    #[cfg(not(any(feature = "lua51", feature = "luajit")))]
    assert_eq!(eval(r#"return tostring(D"7" // 2)"#), "3");
    assert_eq!(eval(r#"return tostring(D"7.5" % 2)"#), "1.5");
    assert_eq!(eval(r#"return tostring(-D"2")"#), "-2");
    assert_eq!(eval(r#"return tostring(1 + D"2")"#), "3");
}

#[test]
fn comparison() {
    assert_eq!(eval(r#"return tostring(D"1.0" == D"1.00")"#), "true");
    assert_eq!(eval(r#"return tostring(D"1" < D"2")"#), "true");
    assert_eq!(eval(r#"return tostring(D"2" <= D"2")"#), "true");
    assert_eq!(eval(r#"return tostring(D"5":eq(5))"#), "true");
    assert_eq!(eval(r#"return tostring(D"4.5":cmp(5))"#), "-1");
    // a decimal is never equal to a Lua number
    assert_eq!(eval(r#"return tostring(D"5" == 5)"#), "false");
}

#[test]
fn a_float_with_a_fraction_is_not_an_operand() {
    fails(r#"return D"1" * 0.1"#, "fraction");
    fails(r#"return D(0.5)"#, "fraction");
    // Luau says "compare", Lua 5.4 reports the bad argument to `__lt`
    assert!(lua().load(r#"return D"5" < 6"#).exec().is_err());
}

#[test]
fn division_by_zero_and_overflow() {
    fails(r#"return D"1" / 0"#, "divide by zero");
    fails(r#"return D"1" % 0"#, "divide by zero");
    fails(
        r#"return D"79228162514264337593543950335" + 1"#,
        "too large",
    );
}

#[test]
fn rounding() {
    assert_eq!(eval(r#"return tostring(D"2.5":round())"#), "2");
    assert_eq!(eval(r#"return tostring(D"3.5":round())"#), "4");
    assert_eq!(eval(r#"return tostring(D"2.5":round(0, "half_up"))"#), "3");
    assert_eq!(
        eval(r#"return tostring(D"-2.5":round(0, "half_up"))"#),
        "-3"
    );
    assert_eq!(eval(r#"return tostring(D"2.345":round(2))"#), "2.34");
    assert_eq!(eval(r#"return tostring(D"1234":round(-2))"#), "1200");
    assert_eq!(eval(r#"return tostring(D"2.1":ceil())"#), "3");
    assert_eq!(eval(r#"return tostring(D"-2.1":floor())"#), "-3");
    assert_eq!(eval(r#"return tostring(D"2.9":trunc())"#), "2");
    fails(
        r#"return D"1":round(0, "sideways")"#,
        "Unknown rounding mode",
    );
}

#[test]
fn scale_normalize_and_conversions() {
    assert_eq!(eval(r#"return tostring(D"1.50":scale())"#), "2");
    assert_eq!(eval(r#"return tostring(D"1.50":normalize())"#), "1.5");
    assert_eq!(eval(r#"return tostring(D"4":is_integer())"#), "true");
    assert_eq!(eval(r#"return tostring(D"-3":sign())"#), "-1");
    assert_eq!(eval(r#"return tostring(D"2.5":to_number())"#), "2.5");
    assert_eq!(eval(r#"return tostring(D(7))"#), "7");
    assert_eq!(eval(r#"return tostring(D(D"1.20"))"#), "1.20");
    fails(r#"return D"abc""#, "Cannot convert");
    fails(r#"return D(true)"#, "Expected a Decimal");
}

#[test]
fn max_min_sum() {
    assert_eq!(eval(r#"return tostring(D.max(D"1.5", 2, D"1.9"))"#), "2");
    assert_eq!(eval(r#"return tostring(D.min(D"1.5", 2, D"1.9"))"#), "1.5");
    assert_eq!(
        eval(r#"return tostring(D.sum({ D"0.1", D"0.2", 3 }))"#),
        "3.3"
    );
    fails(r#"return D.max()"#, "at least one");
}

#[test]
fn rust_side() {
    let lua = lua();
    let v: mlua::Value = lua.load(r#"return D"12.50""#).eval().unwrap();
    let mlua::Value::UserData(u) = v else {
        panic!()
    };
    assert_eq!(mlua_decimal::as_decimal(&u).unwrap().to_string(), "12.50");
    lua.globals()
        .set("x", mlua_decimal::Dec("1.25".parse().unwrap()))
        .unwrap();
    assert_eq!(
        lua.load("return tostring(x * 2)").eval::<String>().unwrap(),
        "2.50"
    );
    assert_eq!(mlua_decimal::from_f64(0.1).unwrap().to_string(), "0.1");
    assert!(mlua_decimal::from_f64(f64::NAN).is_none());
}
