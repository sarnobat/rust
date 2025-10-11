
test:	build
	# run both in parallel for 10 seconds
	timeout 10s cargo run --release --bin cpu_increment | tee cpu_output.txt &
	timeout 10s cargo run --release --bin gpu_increment  | tee gpu_output.txt &
	wait

# Build both CPU and GPU binaries
all: cpu gpu

cpu:
	cargo run --release --bin cpu_increment

gpu:
	cargo run --release --bin gpu_increment

build:
	cargo build --release --workspace

clean:
	cargo clean

