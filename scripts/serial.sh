cargo run --release --bin tablegen -- pattern-free assets/pf.txt
bash scripts/build.sh assets/pf.txt
cargo run --release --bin encoder -- assets/pf.txt assets/test.txt assets/filtered.txt assets/code.txt
bash scripts/build.sh
bash scripts/simulate.sh assets/code.txt assets/decoded.txt
cargo run --release -- assets/filtered.txt assets/decoded.txt