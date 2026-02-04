package main

import "fmt"

func main() {
	for n := range 100 {
		n++
		switch {
		case n%15 == 0:
			fmt.Println("FizzBuzz")
		case n%3 == 0:
			fmt.Println("Fizz")
		case n%5 == 0:
			fmt.Println("Buzz")
		default:
			fmt.Println(n)
		}
	}
}
