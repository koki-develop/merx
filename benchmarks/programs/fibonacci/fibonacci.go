package main

import "fmt"

func main() {
	n := 30
	fmt.Println(0)
	fmt.Println(1)
	a, b := 0, 1
	for i := range n - 2 {
		_ = i
		temp := a + b
		fmt.Println(temp)
		a = b
		b = temp
	}
}
