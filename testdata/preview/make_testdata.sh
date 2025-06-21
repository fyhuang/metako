ffmpeg -y -f lavfi -i testsrc=duration=10:size=320x240 -pix_fmt yuv420p \
    -c:v libx264 -crf 35 -preset veryslow short_video.mp4
