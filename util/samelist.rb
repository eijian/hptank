#/usr/bin/ruby
#

USAGE = "Usage: cat <same list> | samelist.rb <start line> <end line> "


def init
  if ARGV.size != 2
    STDERR.puts USAGE
    exit 1
  end

  st = ARGV.shift.to_i
  ed = ARGV.shift.to_i
  return st, ed
end

def image(im)
  body = <<-EOS
  <table>
    <tr>
      <td>ID:#{im[0]} (#{im[3] / 1000} kB)</td>
    </tr>
    <tr>
      <td>#{im[1]} x #{im[2]} (#{sprintf("%.1f", im[4] / 1000.0)} k / #{sprintf("%2.2f", im[1] * 1.0/im[2])})</td>
    </tr>
    <tr>
      <td>
        <a target="one" href="http://192.168.11.50:4567/imageno/#{im[0]}">
          <img src="http://192.168.11.50:4567/imageno/#{im[0]}" width="300px">
        </a>
      </td>
    </tr>
  </table>
  EOS
  body
end


def line(l)
  imgs = l.split(/\//).map {|l|
    l =~ /(\d+)\(\((\d+), (\d+)\),(\d+)/
    id = $1.to_i
    x  = $2.to_i
    y  = $3.to_i
    sz = $4.to_i
    [id, x, y, sz, x*y]
  }
  body = ""
  imgs.each do |im|
    body += <<-EOS
      <td>
        #{image(im)}
      </td>
    EOS
  end
  body
end

def page(st, ed)
  body = ""
  cnt = 0
  STDIN.each do |l|
    cnt += 1
    next if cnt < st
    break if cnt > ed
    body += <<-EOS
      <tr>
        #{line(l.chomp)}
      </tr>
    EOS
  end
  body
end

def main
  st,ed = init

  puts <<-EOS
    <html>
      <head>
        <title>SAMELIST</title>
      </head>
      <body>
        <div id="title">
          <h2>SAMELIST</h2>
        </div>
        <div>
          <table>
#{page(st, ed)}
          </table>
        </div>
      </body>
    </html>
  EOS

end

main

#---
