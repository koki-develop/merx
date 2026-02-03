# Examples

## Fibonacci

This example prints the Fibonacci sequence up to the `n`-th term, where `n` is provided by user input.

### Flowchart

```mermaid
flowchart TD
    Start([Start]) --> A[print 'Enter a positive integer: ']
    A --> B[n = input as int]
    B --> C{n <= 0?}
    C -->|Yes| D[error 'Input must be a positive integer']
    D -->|exit 1| End([End])
    C -->|No| E{n == 1?}
    E -->|Yes| F[println 0]
    F --> End
    E -->|No| G{n == 2?}
    G -->|Yes| H[println 0; println 1]
    H --> End
    G -->|No| I[a = 0; b = 1]
    I --> J[println 0; println 1]
    J --> K[i = 3]
    K --> L{i <= n?}
    L -->|No| End
    L -->|Yes| M[temp = a + b]
    M --> N[println temp]
    N --> O[a = b]
    O --> P[b = temp]
    P --> Q[i = i + 1]
    Q --> L
```

### Run

```console
$ merx run fibonacci.mmd
Enter a positive integer: 10
0
1
1
2
3
5
8
13
21
34
```

## FizzBuzz

This example prints the numbers from 1 to 100, replacing multiples of 3 with "Fizz", multiples of 5 with "Buzz", and multiples of both with "FizzBuzz".

### Flowchart

```mermaid
flowchart TD
    Start([Start]) --> A[n = 1]
    A --> B{n <= 100?}
    B -->|No| End([End])
    B -->|Yes| C{n % 15 == 0?}
    C -->|Yes| D[println 'FizzBuzz']
    C -->|No| E{n % 3 == 0?}
    E -->|Yes| F[println 'Fizz']
    E -->|No| G{n % 5 == 0?}
    G -->|Yes| H[println 'Buzz']
    G -->|No| I[println n]
    D --> J[n = n + 1]
    F --> J
    H --> J
    I --> J
    J --> B
```

### Run

```console
$ merx run fizzbuzz.mmd
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
...
```
