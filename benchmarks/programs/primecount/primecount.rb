count = 0
(2..10000).each do |n|
  is_prime = true
  d = 2
  while d * d <= n
    if n % d == 0
      is_prime = false
      break
    end
    d += 1
  end
  count += 1 if is_prime
end
puts count
