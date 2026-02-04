total = 0
(1..10000).each do |i|
  x = i
  steps = 0
  while x != 1
    if x % 2 == 0
      x = x / 2
    else
      x = x * 3 + 1
    end
    steps += 1
  end
  total += steps
end
puts total
