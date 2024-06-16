# How to generate source-based coverage for a Rust project
test :
	CARGO_INCREMENTAL=0 RUSTFLAGS=-Cinstrument-coverage LLVM_PROFILE_FILE=cargo-test-%p-%m.profraw cargo test

# gcov How to generate .gcda files for a Rust project
# export CARGO_INCREMENTAL=0
# export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
# export RUSTDOCFLAGS="-Cpanic=abort"

# html report 
html : test
	grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html


# lcov
lcov : test
	grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/tests.lcov

clean :
	rm -rf ./target/debug/deps/*
	rm -rf ./target/coverage/*
	rm -rf ./cargo-test-*.profraw