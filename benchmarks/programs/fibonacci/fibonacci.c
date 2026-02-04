#include <stdio.h>

int main() {
    int n = 30;
    printf("0\n");
    printf("1\n");
    int a = 0, b = 1;
    for (int i = 3; i <= n; i++) {
        int temp = a + b;
        printf("%d\n", temp);
        a = b;
        b = temp;
    }
    return 0;
}
