#include <stdio.h>

int main() {
    int n = 100, s = 0;
    for (int i = 1; i <= n; i++) {
        for (int j = 1; j <= n; j++) {
            int a = i, b = j;
            while (b != 0) {
                int t = b;
                b = a % b;
                a = t;
            }
            s += a;
        }
    }
    printf("%d\n", s);
    return 0;
}
