alias c := check-all

risico-tests:
    cd ./risico/rv-tests && ./run.py

check-all: check-risico check-proggen check-kiwi check-encoding

check-risico:
    cd ./risico                 && cargo check -q

check-proggen:
    cd ./program-gen            && cargo check -q
    
check-kiwi:
    cd ./kiwi/kiwi-rust-codegen && cargo check -q
    cd ./kiwi/kiwi-toml         && cargo check -q

check-encoding:
    cd ./instr-encoding         && cargo check -q