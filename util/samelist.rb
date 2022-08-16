#/usr/bin/ruby
#

USAGE = "Usage: cat <same list> | samelist.rb <start line> <end line> "


def init
  if ARGV.size != 2
    STDERR.puts USAGE
    exit 1
  end

  @nchk = 0
  @discard = Array.new
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
  return "" if l =~ /^\s*$/
  imgs = l.split(/\//).map {|l2|
    l2 =~ /(\d+)\(\((\d+), (\d+)\),(\d+)/
    id = $1.to_i
    x  = $2.to_i
    y  = $3.to_i
    sz = $4.to_i
    [id, x, y, sz, x*y]
  }
  #STDERR.puts "LINE:#{l}"
  imgs = delete_invalid_shape(imgs)
  return "" if imgs == nil || imgs.size == 0

  imgs = if keep_all?(imgs) 
    []
  else
    delete_looser(imgs)
    #is = select_winner(is)
    #is = if is == nil then [] else is end
  end

  if imgs.size >= 2
    @nchk += 1
    #STDERR.puts "NO WINNER: #{imgs.size} / #{l}"
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
    ""
  end
end

def delete_invalid_shape(imgs)
  imgs2 = Array.new
  s1 = (imgs[0][1].to_f / imgs[0][2]) - 1.0 # x / y - 1.0 で縦横比を±に
  imgs.each do |im|
    if im[2] / im[1] >= 3 && im[2] >= 2000
      @discard << im[0]
      next
    end
    s = (im[1].to_f / im[2]) - 1.0
    imgs2 << im if s1 * s >= 0.0
  end
  #STDERR.puts "DELETE INDEX: #{imgs.size - imgs2.size}" if imgs2.size < imgs.size
  imgs2
end

def keep_all?(imgs)
  keep = true
  imgs.each do |im|
    if im[1] < 2000 || im[2] < 2000
      keep = false
      break
    end
  end
  keep
end

def delete_looser(imgs)
  chk = [true] * imgs.size
  imgs.each_with_index do |sim, i|
    #STDERR.puts "SIM(#{i}) = #{sim}"
    next if chk[i] == false
    (i+1).upto(imgs.size - 1) do |j|
      dim = imgs[j]
      #STDERR.puts " DIM(#{j}) = #{dim}"
      if lose?(sim, dim)
        @discard << sim[0]
        chk[i] = false
      elsif lose?(dim, sim)
        @discard << dim[0]
        chk[j] = false
      end
    end
  end
  #STDERR.puts "CHK: #{chk}"
  #STDERR.puts "IMG: #{imgs.size}"
  imgs2 = []
  imgs.zip(chk).each do |e|
    imgs2 << e[0] if e[1] == true
  end
  #STDERR.puts "IMG2: #{imgs2}"
  imgs2
end

def lose?(img1, img2)
  # ＊＊ １が負ける条件 ＊＊
  # １が解像度、ファイルサイズともに２に負けている
  return true if (img1[1] <= img2[1] && img1[2] <= img2[2] && img1[3] <= img2[3])
  # １の面積が２の80%%以下かつ２のサイズが100kB以上
  return true if img1[4] < img2[4] * 0.8 && img2[3] > 100000
  # ２の面積が１の90%以上かつ２のファイルサイズが１の1.25倍以上
  return true if img2[4] > img1[4] * 0.9 && img2[3] > img1[3] * 1.25
  # ２の面積が１より大きく、２のファイルサイズが１の50%以上
  return true if img2[4] > img1[4] && img2[3] > img1[3] * 0.5
  # ２の面積が１より大きく、２のファイルサイズが１の70%以上で200kB以上
  return true if img2[4] > img1[4] && img2[3] > img1[3] * 0.7 && img2[3] > 200000
  # １の両辺がともに1000以下で２の面積が1000k以上、かつ２のファイルサイズが１の50%以上
  return true if img1[1] <= 1000 && img1[2] <= 1000 && img2[4] > 1000000 && img2[3] > img1[3] * 0.5
     #(img2[4] < img1[4] && img2[4] * 100.0 / img1[4] > 80.0 && img2[3] * 100.0 / img1[3] > 1.5) ||
     #(img1[4] * 100.0 / img2[4] < 80.0 && (img2[3] > 100000 || img1[3] * 100.0 / img2[3] < 150)) ||
     #(img2[4] * 100.0 / img1[4] > 150)
  false
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

  STDERR.puts "#CHECK=#{@nchk}"
end

main

#---
