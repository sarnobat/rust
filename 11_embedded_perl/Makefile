export PERL5LIB := /opt/homebrew/opt/perl/lib/perl5/5.40:/opt/homebrew/opt/perl/lib/perl5/site_perl/5.40
export MACOSX_DEPLOYMENT_TARGET := 14.0

build:
	cargo run


embed:
	cd /tmp/
	# 1. Fetch and unpack a vanilla Perl
	curl -LO https://www.cpan.org/src/5.0/perl-5.40.0.tar.gz && \
	tar xf perl-5.40.0.tar.gz && cd perl-5.40.0 && 	./Configure -des \
		-Dprefix=/tmp/perl_embed \
		-Duseshrplib \
		-Dcc=clang \
		-Accflags='-mmacosx-version-min=14.0' \
		-Aldflags='-mmacosx-version-min=14.0' \
		&&	make -j"$(sysctl -n hw.ncpu)" \
	&& make install
