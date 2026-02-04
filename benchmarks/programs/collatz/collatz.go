package main

import "fmt"

func main() {
	total := 0
	for i := range 10000 {
		i++
		x := i
		steps := 0
		for x != 1 {
			if x%2 == 0 {
				x = x / 2
			} else {
				x = x*3 + 1
			}
			steps++
		}
		total += steps
	}
	fmt.Println(total)
}
