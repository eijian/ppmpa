#/usr/bin/ruby

def to_num(s)
  if s =~ /\((\S+)\)$/
    return $1.to_f
  else
    return s.to_f
  end
end

STDIN.each do |l|
  l.chomp =~ /\((.+),\(Vector3 (\S+) (\S+) (\S+),Vector3 (\S+) (\S+) (\S+)\)\)/
  if $1 == nil
    puts l
  else
    wl = $1
    v1x = to_num($2)
    v1y = to_num($3)
    v1z = to_num($4)
    v2x = to_num($5)
    v2y = to_num($6)
    v2z = to_num($7)
    #puts "PHOTON[WL:#{$1},RAY[V3[#{v1x},#{v1y},#{v1z}],V3[#{v2x},#{v2y},#{v2z}]]]"
    puts "#{$1} #{v1x} #{v1y} #{v1z} #{v2x} #{v2y} #{v2z}"
  end
end


