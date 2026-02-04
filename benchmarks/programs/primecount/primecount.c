#include <stdio.h>
#include <stdbool.h>

int main() {
    int count = 0;
    for (int n = 2; n <= 10000; n++) {
        bool is_prime = true;
        for (int d = 2; d * d <= n; d++) {
            if (n % d == 0) {
                is_prime = false;
                break;
            }
        }
        if (is_prime)
            count++;
    }
    printf("%d\n", count);
    return 0;
}
