fn main() {
    let n = 30;
    println!("0");
    println!("1");
    let (mut a, mut b) = (0_i64, 1_i64);
    for _ in 3..=n {
        let temp = a + b;
        println!("{}", temp);
        a = b;
        b = temp;
    }
}
