export PERL5LIB := /opt/homebrew/opt/perl/lib/perl5/5.40:/opt/homebrew/opt/perl/lib/perl5/site_perl/5.40
export MACOSX_DEPLOYMENT_TARGET := 14.0

build:
	cargo run


embed:
	cd /tmp/
	# 1. Fetch and unpack a vanilla Perl
	curl -LO https://www.cpan.org/src/5.0/perl-5.40.0.tar.gz && \
	tar xf perl-5.40.0.tar.gz && cd perl-5.40.0 && make distclean \
	&& \
	./Configure -des \
  -Dprefix=/tmp/perl_static \
  -Duserelocatableinc \
  -Dusedl=no \
  -Uuseshrplib \
  -Dcc=clang \
  -Accflags='-fPIC -mmacosx-version-min=14.0' \
  -Aldflags='-mmacosx-version-min=14.0' \
  -Dextensions='List/Util IO Fcntl Data/Dumper Hash/Util' \
  -Dstatic_ext='List/Util IO Fcntl Data/Dumper Hash/Util' \
  -Dnoextensions='Encode Compress::Raw::Zlib Compress::Raw::Bzip2 Filter::Util::Call' \
  -Ulocincpth= -Uloclibpth= \
	&& make \
	&& ls -l \
		lib/auto/List/Util/Util.a \
		lib/auto/Hash/Util/Util.a \
		lib/auto/IO/IO.a \
		lib/auto/Fcntl/Fcntl.a \
		lib/auto/Data/Dumper/Dumper.a \
	&& make install
