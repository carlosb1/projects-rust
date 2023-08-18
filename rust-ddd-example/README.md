## Command for streaming source video

```ffmpeg -re -i hawkeye.mp4 -vcodec libvpx -an  -cpu-used 5 -deadline 1 -g 10 -error-resilient 1 -auto-alt-ref 1 -f rtp rtp://127.0.0.1:5004?pkt_size=1200```

```ffmpeg -re -i hawkeye.mp4 -c:v copy -an -f rtp "rtp://192.168.0.130:5004" -vn -acodec copy -f rtp "rtp://192.168.0.130:50043" -sdp_file video.sdp ```
