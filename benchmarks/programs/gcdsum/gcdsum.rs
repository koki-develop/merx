fn main() {
    let n: i64 = 100;
    let mut s: i64 = 0;
    for i in 1..=n {
        for j in 1..=n {
            let (mut a, mut b) = (i, j);
            while b != 0 {
                let temp = b;
                b = a % b;
                a = temp;
            }
            s += a;
        }
    }
    println!("{}", s);
}
