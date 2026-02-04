count = 0
for n in range(2, 10001):
    is_prime = True
    d = 2
    while d * d <= n:
        if n % d == 0:
            is_prime = False
            break
        d += 1
    if is_prime:
        count += 1
print(count)
