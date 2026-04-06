::合并多张图片到一张图片
::-geometry +0 表示没有边框，维持原有图片大小。
::-tile 1x 表示一列排列
magick.exe montage source-[0-32].png -geometry +0 -tile 1x target.png 