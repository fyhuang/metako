TESTSRC_ARGS_VIDONLY="-f lavfi -i testsrc -pix_fmt yuv420p"
TESTSRC_ARGS_VIDAUD="-f lavfi -i testsrc -f lavfi -i sine=frequency=440 -pix_fmt yuv420p"

#ffmpeg -y $TESTSRC_ARGS_VIDONLY \
#    -c:v libx264 -crf 35 -preset veryfast -t 10 vidonly_h264.mp4

# H.264 + AAC in MP4 container. Chrome can play this natively.
ffmpeg -y $TESTSRC_ARGS_VIDAUD \
    -c:v libx264 -crf 35 -preset veryfast \
    -c:a aac -b:a 24k \
    -t 10 vidaud_h264_aac.mp4

# H.265 + AAC in MKV container. Chrome can't play this natively.
ffmpeg -y $TESTSRC_ARGS_VIDAUD \
    -c:v libx265 -crf 35 -preset veryfast \
    -c:a aac -b:a 24k \
    -t 10 vidaud_h265_aac.mkv

