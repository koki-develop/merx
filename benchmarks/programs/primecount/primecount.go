package main

import "fmt"

func main() {
	count := 0
	for n := 2; n <= 10000; n++ {
		isPrime := true
		for d := 2; d*d <= n; d++ {
			if n%d == 0 {
				isPrime = false
				break
			}
		}
		if isPrime {
			count++
		}
	}
	fmt.Println(count)
}
