let total = 0;
for (let i = 1; i <= 10000; i++) {
  let x = i;
  let steps = 0;
  while (x !== 1) {
    if (x % 2 === 0) {
      x = Math.floor(x / 2);
    } else {
      x = x * 3 + 1;
    }
    steps++;
  }
  total += steps;
}
console.log(total);
