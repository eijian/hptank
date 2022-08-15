#/usr/bin/ruby
#

USAGE = "Usage: cat <same list> | samelist.rb <start line> <end line> "


def init
  if ARGV.size != 2
    STDERR.puts USAGE
    exit 1
  end

  @nwin = 0
  st = ARGV.shift.to_i
  ed = ARGV.shift.to_i
  return st, ed
end

def image(im)
  body = <<-EOS
  <div class="card">
    <div class="card-content">
      <p class="title is-6">ID:#{im[0]} (#{im[3] / 1000} kB)</p>
      <p class="subtitle is-7">#{im[1]} x #{im[2]} (#{sprintf("%.1f", im[4] / 1000.0)} k / #{sprintf("%2.2f", im[1] * 1.0/im[2])})</p>
    </div>
    <div class="card-image">
      <figure class="image">
        <a target="one" href="http://192.168.11.50:4567/imageno/#{im[0]}">
          <img src="http://192.168.11.50:4567/imageno/#{im[0]}">
        </a>
      </figure>
    </div>
  </div>
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
  winner = select_winner(imgs)

  if winner == nil
    body = "      <div class=\"columns\">\n"
    imgs.each do |im|
      body += <<-EOS
        <div class="column">
          #{image(im)}
        </div>
      EOS
    end
    body + "      </div>\n"
  else
    @nwin += 1
    ""
  end
end

def select_winner(imgs)
  winner = nil
  xs = []
  ys = []
  szs = []
  imgs.each do |i|
    xs << i[1]
    ys << i[2]
    szs << i[3]
  end
  mx = xs.max
  my = ys.max
  msz = szs.max

  imgs.each do |chlg|
    if chlg[1] == mx && chlg[2] == my
      if chlg[3] == msz
        winner = chlg
      elsif chlg[3] * 100.0 / msz > 90
        winner = chlg
      end
    end
  end
  winner
end

def page(st, ed)
  body = ""
  cnt = 0
  STDIN.each do |l|
    cnt += 1
    next if cnt < st
    break if cnt > ed
    body += <<-EOS
        #{line(l.chomp)}
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
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css">
      </head>
      <body>
      <div class="container">
        <section class="hero">
          <div class="hero-body">
            <p class="title">
              SAMELIST
            </p>
          </div>
        </section>
        <section>
          #{page(st, ed)}
        </section>
      </div>
      </body>
    </html>
  EOS

  STDERR.puts "#WINNER=#{@nwin}"
end

main

#---
