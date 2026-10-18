-- An exact base-10 number (up to 28 digits).
-- Arithmetic (+ - * / // %) and ordering (< <= ==) work between two Decimals; an integer may stand in for either
-- operand. A Luau number with a fraction is an error in arithmetic (write D"0.1"), and `d < 5` and `d == 5`
-- do not work either (Luau never asks a userdata about a number): use d:lt(5), d:eq(5) or d:cmp(5).
declare extern type Decimal with
	function __add(self, other: Decimal | number): Decimal
	function __sub(self, other: Decimal | number): Decimal
	function __mul(self, other: Decimal | number): Decimal
	function __div(self, other: Decimal | number): Decimal
	function __idiv(self, other: Decimal | number): Decimal
	function __mod(self, other: Decimal | number): Decimal
	function __unm(self): Decimal
	function __eq(self, other: Decimal): boolean
	function __lt(self, other: Decimal): boolean
	function __le(self, other: Decimal): boolean
	-- -1, 0 or 1; the other operand is a Decimal or an integer
	function cmp(self, other: Decimal | number): number
	function eq(self, other: Decimal | number): boolean
	function lt(self, other: Decimal | number): boolean
	function le(self, other: Decimal | number): boolean
	-- digits may be negative (round to tens); mode is half_even (the default), half_up, half_down, down, up, floor or ceil
	function round(self, digits: number?, mode: string?): Decimal
	function floor(self): Decimal
	function ceil(self): Decimal
	function trunc(self): Decimal
	function abs(self): Decimal
	function sign(self): number
	-- the digits after the decimal point: D"1.50":scale() is 2
	function scale(self): number
	-- without trailing zeros: D"1.50":normalize() is 1.5
	function normalize(self): Decimal
	function is_integer(self): boolean
	-- the explicit, lossy way out to a Luau number
	function to_number(self): number
end

-- D"12.50", D(3) or D(d): a Decimal from its text, an integer or a Decimal (a float with a fraction is refused).
-- It takes `any` on purpose: `local price = D(row.Price)` types a cell, a variable or an `evaluate` result, which the
-- checker cannot see into, so that `price < 5` and `price == 5` are reported before the script runs.
declare D: ((v: any) -> Decimal) & {
	max: (...(Decimal | number)) -> Decimal,
	min: (...(Decimal | number)) -> Decimal,
	sum: ({ Decimal | number }) -> Decimal,
}
