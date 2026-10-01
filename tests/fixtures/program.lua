local function counter(start)
    return function(step)
        start = start + step
        return start
    end
end

local function sum(...)
    local total = 0
    for _, value in ipairs({...}) do
        total = total + value
    end
    return total
end

local object = { value = 2, text = "ação", enabled = true }
function object:add(value)
    self.value = self.value + value
    return self.value
end

local total = 0
for i = 1, 10 do
    if i % 2 == 0 then
        total = total + i
    end
end
local i = 0
while i < 3 do
    i = i + 1
end
repeat
    i = i - 1
until i == 0

local next_value = counter(10)
assert(next_value(2) == 12)
assert(next_value(3) == 15)
assert(sum(1, 2, 3, 4) == 10)
assert(object:add(5) == 7)
assert(total == 30 and i == 0)
assert(2 ^ 3 == 8 and -3 % 2 == 1)
assert(#"a\000b" == 3)
assert(object.enabled and not false)
print("Lua 5.1 OK", object.text, total, sum(1, 2, 3))
