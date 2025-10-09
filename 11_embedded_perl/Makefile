export PERL5LIB := /opt/homebrew/opt/perl/lib/perl5/5.40:/opt/homebrew/opt/perl/lib/perl5/site_perl/5.40
export MACOSX_DEPLOYMENT_TARGET := 14.0

build:
	cargo run
download:
	cd /tmp && \
	rm -rf perl-5.40* && \
	curl -LO https://www.cpan.org/src/5.0/perl-5.40.0.tar.gz && \
	tar xf perl-5.40.0.tar.gz

embed: download
	cd /private/tmp/perl-5.40.0 && \
	sh Configure -des \
		-Dprefix=/private/tmp/perl_static \
		-Dusedl=false -Duseshrplib=false \
		-Dstatic_ext='List/Util Data/Dumper Fcntl IO Encode Cwd re File/Glob' \
		-Dcc=clang -Dld=clang \
		-Dccflags='-fno-common -mmacosx-version-min=14.0 -fstack-protector-strong' \
		-Dldflags='-mmacosx-version-min=14.0' \
		-Dusethreads && \
	make miniperl && \
	rm -f ext.libs lib/ExtUtils/Miniperl.pm perlmain.c && \
	./miniperl -Ilib configpm \
	./miniperl -Ilib -MExtUtils::MakeMaker -e 'unlink("Makefile"); unlink "ext.libs"; system("make -f Makefile.SH Makefile");' && \
	echo "5.040000" > version \
	make -j$$(sysctl -n hw.ncpu) && \
	make install



embed3: download
	cd /private/tmp/perl-5.40.0 \
	&& sh Configure -des \
  -Dprefix=/private/tmp/perl_static \
  -Duseshrplib=false \
  -Duserelocatableinc=false \
  -Dusedl=false \
  -Dusethreads \
  -Dstatic_ext='cpan/Scalar-List-Utils ext/Data-Dumper ext/Fcntl ext/IO cpan/Encode' \
  -Dcc=clang -Dld=clang \
  -Dccflags='-fno-common -fstack-protector-strong -mmacosx-version-min=14.0' \
  -Dldflags='-mmacosx-version-min=14.0' \
  && ./miniperl -Ilib -MExtUtils::MakeMaker -e 'unlink("Makefile"); unlink "ext.libs"; system("make -f Makefile.SH Makefile");' \
 	&& make	\
 	&& make install

embed2:	download
	cd perl-5.40.0 && make distclean 
	&& 	./Configure -des \
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
	&& ar q libperl.a lib/auto/List/Util/Util.a lib/auto/Hash/Util/Util.a `xcrun -f ranlib` libperl.a \
	&& cd - \
	&& make install

link:
link:
	cd /private/tmp/perl_static/lib/5.40.0/darwin-2level && \
	rm -rf tmp_obj && mkdir -p tmp_obj && \
	cd tmp_obj && \
	ar x /private/tmp/perl_static/lib/5.40.0/darwin-2level/auto/Hash/Util/Util.a && \
	ar x /private/tmp/perl_static/lib/5.40.0/darwin-2level/auto/List/Util/Util.a && \
	echo "Extracted object files:" && ls -1 *.o && \
	cd /private/tmp/perl_static/lib/5.40.0/darwin-2level/CORE && \
	ar q libperl.a /private/tmp/perl_static/lib/5.40.0/darwin-2level/tmp_obj/*.o && \
	`xcrun -f ranlib` libperl.a && \
	echo "✅ libperl.a updated with List::Util + Hash::Util objects." && \
	nm libperl.a | grep XS_List__Util_bootstrap || true


# 	cd /private/tmp/perl_static/lib/5.40.0/darwin-2level/ \
# 	&& ls -l \
# 		auto/List/Util/Util.a \
# 		auto/Hash/Util/Util.a \
# 		auto/IO/IO.a \
# 		auto/Fcntl/Fcntl.a \
# 		auto/Data/Dumper/Dumper.a \
# 	&& sudo chmod -R 777 /private/tmp/perl_static \
# 	&& cd /private/tmp/perl_static/lib/5.40.0/darwin-2level \
# 	&& mkdir -p tmp_obj \
# 	&& cd tmp_obj \
# 	&& ar x ../auto/Hash/Util/Util.a \
# 	&& ar x ../auto/List/Util/Util.a \
# 	&& echo "Extracted:" && ls -l *.o \
# 	&& cd ../CORE \
# 	&& ar q libperl.a ../tmp_obj/*.o \
# 	&& `xcrun -f ranlib` /private/tmp/perl_static/lib/5.40.0/darwin-2level/CORE/libperl.a \
# 	&& echo "Updated libperl.a with List::Util + Hash::Util objects." \
# 	&& nm libperl.a | grep XS_List__Util_bootstrap || true \
# 	&& cd /tmp/perl-5.40.0 \
# 	&& make install
