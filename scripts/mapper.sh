#!/bin/bash

index=$1
table="build/c42.txt"
corpus="assets/corpus-$index.txt"
filter="assets/filter-$index.txt"
encoded="assets/encoded-$index.txt"
decoded="assets/decoded-$index.txt"
result="assets/result-$index.json"
cargo run --release --bin encoder -- $table $corpus $filter $encoded
bash scripts/simulate.sh $encoded $decoded
cargo run --release -- $table $decoded $filter $result
