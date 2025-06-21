ffmpeg -i ~/Downloads/art001m1203451716~small.mp4 \
    -c:v libx264 -ss 00:10:00 -t 00:00:10 -crf 35 -preset veryslow \
    -c:a aac -b:a 16k \
    art001m1203451716~small_10s_nosub.mp4

ffmpeg -i art001m1203451716~small_10s_nosub.mp4 \
    -i en.srt -i jp.srt \
    -map 0 -map 1 -map 2 \
    -c:v copy -c:a copy \
    -c:s mov_text \
    -metadata:s:s:0 language=eng -metadata:s:s:0 handler_name=English -metadata:s:s:0 title=English \
    -metadata:s:s:1 language=jpn -metadata:s:s:1 handler_name=Japanese -metadata:s:s:1 title=Japanese \
    art001m1203451716~small_10s.mp4
