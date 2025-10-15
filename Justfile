# Compute platform triple
_platform_triple := `rustc --print target-list | grep $(rustc --version --verbose | grep host | cut -d' ' -f2) | head -n1`

BIN_NAME := "whas-" + _platform_triple

build:
	-rm dist/whas

	cargo clean --release

	cargo build \
		--release \
		--out-dir ./dist \
		-Z unstable-options

	mv dist/whale-schema dist/{{BIN_NAME}}

	# should be done automatically, see Cargo.toml
	strip dist/{{BIN_NAME}}

	# compress executable
	# sadly doesnt seem to work on Mac13+ anymore:
	# https://github.com/upx/upx/issues/612
	#upx --best --lzma dist/whas

install-upx:
	brew install upx

# find target triple for this platform
# will be 'aarch64-apple-darwin' for M*
rust-meta:
	rustc -vV

platform-triple:
	@rustc --print target-list | grep $(rustc --version --verbose | grep host | cut -d' ' -f2) | head -n1
