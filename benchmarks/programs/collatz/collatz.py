total = 0
for i in range(1, 10001):
    x = i
    steps = 0
    while x != 1:
        if x % 2 == 0:
            x = x // 2
        else:
            x = x * 3 + 1
        steps += 1
    total += steps
print(total)
