n = 30
print(0)
print(1)
a, b = 0, 1
for i in range(3, n + 1):
    temp = a + b
    print(temp)
    a = b
    b = temp
