#include <stdio.h>
#include <stdlib.h>

int main() {
    int len = 0;
    char *s = malloc(1);
    s[0] = '\0';
    for (int i = 0; i < 10000; i++) {
        s = realloc(s, len + 2);
        s[len] = 'a';
        s[len + 1] = '\0';
        len++;
    }
    printf("%d\n", len);
    free(s);
    return 0;
}
