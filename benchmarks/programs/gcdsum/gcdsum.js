const n = 100;
let s = 0;
for (let i = 1; i <= n; i++) {
  for (let j = 1; j <= n; j++) {
    let a = i, b = j;
    while (b !== 0) {
      const temp = b;
      b = a % b;
      a = temp;
    }
    s += a;
  }
}
console.log(s);
