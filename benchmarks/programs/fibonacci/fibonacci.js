const n = 30;
console.log(0);
console.log(1);
let a = 0,
  b = 1;
for (let i = 3; i <= n; i++) {
  const temp = a + b;
  console.log(temp);
  a = b;
  b = temp;
}
