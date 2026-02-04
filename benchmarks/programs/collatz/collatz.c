#include <stdio.h>

int main() {
    long long total = 0;
    for (int i = 1; i <= 10000; i++) {
        long long x = i;
        int steps = 0;
        while (x != 1) {
            if (x % 2 == 0)
                x /= 2;
            else
                x = x * 3 + 1;
            steps++;
        }
        total += steps;
    }
    printf("%lld\n", total);
    return 0;
}
