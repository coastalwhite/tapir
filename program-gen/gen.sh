#!/bin/sh

nr_tests=100
# min_bbs=(1 10)
# diff_bbs=(1 10 100)
# min_instrs_per_bb=(1 10)
# diff_instrs_per_bb=(1 10 100)
min_bbs=(1 10 100 1000 10000)
diff_bbs=(1)
min_instrs_per_bb=(1 10 100)
diff_instrs_per_bb=(1)
out_file="out.csv"

set -e
set -x

cargo build --release
echo 'total_num_instructions,avg_num_instructions,total_time,avg_time,nr_tests,min_bbs,max_bbs,min_instrs_per_bb,max_instrs_per_bb' > $out_file

for min_bb in "${min_bbs[@]}"
do
	for diff_bb in "${diff_bbs[@]}"
	do
		max_bb=$(($min_bb+$diff_bb))
		for min_instrs in "${min_instrs_per_bb[@]}"
		do
			for diff_instrs in "${diff_instrs_per_bb[@]}"
			do
				max_instrs=$(($min_instrs+$diff_instrs))

				./target/release/rvhwfuzzer-program-gen speed $nr_tests $min_bb $max_bb $min_instrs $max_instrs >> $out_file 2> err
			done
		done
	done
done
