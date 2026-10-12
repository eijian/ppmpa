#!/usr/bin/ruby
#
# blurred test
#

USAGE = "blurredtest.rb <N> <#samples> [<divisions>]"
DIVS = 64

def init
  if 1 <= ARGV.size && ARGV.size <= 3
    @nval = ARGV[0].to_f
    @nsamp = ARGV[1].to_i
    @divs = if ARGV.size == 3 then ARGV[2].to_i else DIVS end
  else
    STDERR.puts USAGE
    exit 1
  end
end

def f(x)
  #c = Math.cos((x ** 0.5) * Math::PI / 2.0)
  c = 1.0 - x
  c ** (1.0 / (@nval + 1.0))
  #c ** (1.0 / (@nval * (@nval + 1.0)))
end

def blurred_angle
  xi = Random.rand(0.0 .. 1.0)
  cos = f(xi)
  Math.acos(cos)
end

def main
  init

  freq = Array.new(@divs + 1, 0)

  @nsamp.times do
    theta = blurred_angle
    #puts "Theta: #{theta}"
    nth = (theta * 2.0 / Math::PI * @divs.to_f).floor
    #nth = ((theta) * @divs.to_f).floor
    #puts "NTH: #{nth}"
    freq[nth] += 1
  end

  #freq1 = freq.map{|n| n.to_f  / @nsamp * @divs.to_f}
  #puts "F0: #{freq[0]}"
  #puts "#{@nval},#{freq.join(',')}"
  #freq2 = freq.map{|n| n.to_f / freq[0].to_f}
  freq2 = freq.map{|n| n.to_f}

  # 関数演算
  mag = Math::PI / 2.0 / @divs.to_f
  f0 = 1.0
  @divs.times do |i|
    h = Math.cos(i * mag) - Math.cos((i + 1.0) * mag)
    s = 2 * Math::PI * h
    f0 = if i == 0 then freq2[i] / s else f0 end
    puts "(#{i}) N: #{freq2[i]} / N/S: #{freq2[i] / s / f0} / COS: #{Math.cos(i * mag)} / COS2: #{Math.cos(i * mag) ** @nval}"
  end

end


main

#---
