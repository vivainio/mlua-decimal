--!strict
local price = D"12.50"
local qty = D(3)
local total: Decimal = price * qty
local cheap = price < D(20)
local same = price == D"12.5"
local up = total:round(1, "half_up")
local biggest = D.max(price, qty, 7)
local sum = D.sum({ price, qty })
print(tostring(total), cheap, same, up, biggest, sum, price:cmp(5), price:lt(20))
