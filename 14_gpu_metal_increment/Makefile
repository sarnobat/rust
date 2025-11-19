SECS := 2s

# copy-paste friendly
test:
	timeout 2s ./cpu_increment.bin
	timeout 2s ./cpu_multithreaded_increment.bin
	timeout 2s ./gpu_increment.bin
	wait

test_uncompiled:	build
	# run in parallel for 10 seconds
	timeout $(SECS) cargo run --release --bin cpu_increment | tee cpu_output.txt &
	timeout $(SECS) cargo run --release --bin cpu_multithreaded_increment | tee cpu__multithreaded_output.txt &
	timeout $(SECS) cargo run --release --bin gpu_increment  | tee gpu_output.txt &
	wait

all: cpu cpu_multithreaded gpu

cpu:
	cargo run --release --bin cpu_increment

cpu_multithreaded:
	cargo run --release --bin cpu_multithreaded

gpu:
	cargo run --release --bin gpu_increment

build:
	cargo build --release --workspace

clean:
	cargo clean

