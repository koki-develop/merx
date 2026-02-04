n = 100
s = 0
for i in range(1, n + 1):
    for j in range(1, n + 1):
        a, b = i, j
        while b != 0:
            a, b = b, a % b
        s += a
print(s)
