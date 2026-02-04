package main

import "fmt"

func main() {
	s := ""
	for range 10000 {
		s = s + "a"
	}
	fmt.Println(len(s))
}
