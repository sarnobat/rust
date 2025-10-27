all:	run	visualize

build:
	cargo build
	cp /Volumes/git/github/depot_parent/rust/20_exif_to_csv/target/debug/exif2csv .
run:
	cp out.csv out.csv.`date -I`
	cat /Volumes/git/db.git/2021/exif/exif_all_oneline.json | cargo run | tee out.csv | tee /tmp/out.csv | uniq
	
	
visualize:
	/Volumes/git/github/depot_parent/cpp/08_opengl_particles_csv_3d/main /tmp/out.csv