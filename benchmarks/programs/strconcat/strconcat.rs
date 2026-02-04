fn main() {
    let mut s = String::new();
    for _ in 0..10000 {
        s = s + "a";
    }
    println!("{}", s.len());
}
