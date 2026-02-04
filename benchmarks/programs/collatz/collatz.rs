fn main() {
    let mut total: i64 = 0;
    for i in 1..=10000 {
        let mut x: i64 = i;
        let mut steps: i64 = 0;
        while x != 1 {
            if x % 2 == 0 {
                x /= 2;
            } else {
                x = x * 3 + 1;
            }
            steps += 1;
        }
        total += steps;
    }
    println!("{}", total);
}
