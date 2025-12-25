
all: 
	cargo build && crontab -l | target/debug/croncheck 2>(tee /tmp/out.err)
	//cat /tmp/out.err
	ls /tmp/out.err
