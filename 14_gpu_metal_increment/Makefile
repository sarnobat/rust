
test:	build
	# run both in parallel for 10 seconds
	timeout 10s cargo run --release --bin cpu_increment | tee cpu_output.txt &
	timeout 10s cargo run --release --bin cpu_multithreaded_increment | tee cpu__multithreaded_output.txt &
	timeout 10s cargo run --release --bin gpu_increment  | tee gpu_output.txt &
	wait

# Build both CPU and GPU binaries
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

