run:	target/debug/croncheck
	crontab -l | target/debug/croncheck

target/debug/croncheck:
	cargo build

build: 
	cargo build && ln -sf target/debug/croncheck
	#cat /tmp/out.err
	ls /tmp/out.err
