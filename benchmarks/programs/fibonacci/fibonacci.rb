n = 30
puts 0
puts 1
a, b = 0, 1
(3..n).each do
  temp = a + b
  puts temp
  a = b
  b = temp
end
