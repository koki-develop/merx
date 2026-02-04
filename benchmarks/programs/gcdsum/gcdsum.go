package main

import "fmt"

func main() {
	n := 100
	s := 0
	for i := 1; i <= n; i++ {
		for j := 1; j <= n; j++ {
			a, b := i, j
			for b != 0 {
				a, b = b, a%b
			}
			s += a
		}
	}
	fmt.Println(s)
}
