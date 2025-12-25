run:	build
	crontab -l | target/debug/croncheck

build: 
	cargo build && ln -sf target/debug/croncheck
	#cat /tmp/out.err
	ls /tmp/out.err
