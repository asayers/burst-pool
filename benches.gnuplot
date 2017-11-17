set terminal pngcairo enhanced truecolor size 936,520 fontscale 0.8
set output "benches.png"
file = "benches.dat"
set xrange [0:30000]
set xlabel "Latency (μs)"
set format x "%.0s"
set yrange [0:0.0006]
set grid mxtics xtics
set xtics 10000
set mxtics 5
unset ytics
stats file using 0 nooutput
set multiplot layout 3,2 # title "Kernel density estimates of msg-sending latency" font ",14"
plot for [IDX=0:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=1:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=2:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=3:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=4:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
plot for [IDX=5:(STATS_blocks - 1):6] file index IDX smooth kdensity title columnheader(1)
