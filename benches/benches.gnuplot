set terminal pngcairo enhanced truecolor size 720,1400 fontscale 0.8
set output "benches.png"
file = "benches.dat"
set xrange [0:30000]
set xlabel "Latency (μs)"
set format x "%.0s"
set yrange [0:0.001]
stats file using 0 nooutput
set multiplot layout 4, 1 title "Benchmarks" font ",14"
plot for [IDX=0:(STATS_blocks - 1):4] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=1:(STATS_blocks - 1):4] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=2:(STATS_blocks - 1):4] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=3:(STATS_blocks - 1):4] file index IDX smooth kdensity title columnheader(1)
