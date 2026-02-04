n = 100
s = 0
(1..n).each do |i|
  (1..n).each do |j|
    a, b = i, j
    while b != 0
      a, b = b, a % b
    end
    s += a
  end
end
puts s
