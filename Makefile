APP = otto

.PHONY: build
build:
	cargo build --release

.PHONY: run
run:
	cargo run -- --config $(APP).toml --log-level info

.PHONY: lint
lint:
	cargo clippy --workspace --tests --all-features -- -D warnings

.PHONY: test
test:
	cargo test

.PHONY: cover
cover:
	docker run \
		--security-opt seccomp=unconfined \
		-v ${PWD}:/volume \
		xd009642/tarpaulin \
		cargo tarpaulin --out Html --output-dir ./target
	open target/tarpaulin-report.html

.PHONY: cross
cross: build
	docker build -t $(APP)/cross -f cross.Dockerfile .
	docker run -it --rm \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v $$PWD:/src/$(APP) \
		-w /src/$(APP) \
		$(APP)/cross \
		/bin/bash -c ' \
			cross build --target x86_64-unknown-linux-gnu --release && \
			cross build --target x86_64-unknown-linux-musl --release && \
			cross build --target armv7-unknown-linux-gnueabihf --release \
		'

VERSION := $(shell cargo metadata -q | jq -r '.packages[] | select(.name == "$(APP)") | .version')
MACOS_X86_64 := target/package/$(APP)-x86_64-apple-darwin-$(VERSION).zip
LINUX_DYN_X86_64 := target/package/$(APP)-x86_64-unknown-linux-gnu-$(VERSION).tar.gz
LINUX_STAT_X86_64 := target/package/$(APP)-x86_64-unknown-linux-musl-$(VERSION).tar.gz
LINUX_ARMV7 := target/package/$(APP)-armv7-unknown-linux-gnueabihf-$(VERSION).tar.gz

.PHONY: release
release: cross
	@echo "[release] Cleaning up before packaging..."
	mkdir -p target/package
	rm -f $(MACOS_X86_64) $(LINUX_X86_64) $(LINUX_ARMV7)
	@echo "[release] Creating package for MacOS x86_64..."
	zip -j $(MACOS_X86_64) target/release/otto LICENSE README.md
	zip -r $(MACOS_X86_64) examples
	@echo "[release] Creating package for Linux (dynamic) x86_64..."
	tar -cvzf $(LINUX_DYN_X86_64) \
		-C $$PWD/target/x86_64-unknown-linux-gnu/release otto \
		-C $$PWD LICENSE README.md examples
	@echo "[release] Creating package for Linux (static) x86_64..."
	tar -cvzf $(LINUX_STAT_X86_64) \
		-C $$PWD/target/x86_64-unknown-linux-musl/release otto \
		-C $$PWD LICENSE README.md examples
	@echo "[release] Creating package for Linux armv7..."
	tar -cvzf $(LINUX_ARMV7) \
		-C $$PWD/target/armv7-unknown-linux-gnueabihf/release otto \
		-C $$PWD LICENSE README.md examples

# build (default), debian or alpine
IMAGE := build

.PHONY: docker
docker:
	docker build -t $(APP)/$(IMAGE) \
		--build-arg APP_NAME=$(APP) \
		--build-arg VERSION=$(VERSION) \
		-f $(IMAGE).Dockerfile \
		.
	docker run -it --rm \
		-p 9999:9999 \
		-v $$PWD/$(APP).toml:/etc/$(APP)/$(APP).toml \
		-v $$PWD/examples/check_ssl_cert.sh:/usr/local/bin/examples/check_ssl_cert.sh \
		$(APP)/$(IMAGE)

.PHONY: alpine
alpine:
	make docker IMAGE=alpine

.PHONY: debian
debian:
	make docker IMAGE=debian
