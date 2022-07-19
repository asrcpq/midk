mkdir -p /tmp/midk_ksmp
cd /tmp/midk_ksmp
sox -n -r 48000 a1_100.flac synth 10 pluck 55
sox -n -r 48000 a1_50.flac synth 10 sine 55 vol -10dB
sox -n -r 48000 a3_100.flac synth 10 pluck 220
sox -n -r 48000 a3_50.flac synth 10 sine 220 vol -10dB
