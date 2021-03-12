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
			cross build --target armv7-unknown-linux-gnueabihf --release \
		'

VERSION := $(shell cargo metadata -q | jq -r '.packages[] | select(.name == "$(APP)") | .version')
MACOS_X86_64 := target/package/$(APP)-x86_64-apple-darwin-$(VERSION).zip
LINUX_X86_64 := target/package/$(APP)-x86_64-unknown-linux-gnu-$(VERSION).tar.gz
LINUX_ARMV7 := target/package/$(APP)-armv7-unknown-linux-gnueabihf-$(VERSION).tar.gz

.PHONY: release
release: cross
	@echo "[release] Cleaning up before packaging..."
	mkdir -p target/package
	rm -f $(MACOS_X86_64) $(LINUX_X86_64) $(LINUX_ARMV7)
	@echo "[release] Creating package for MacOS x86_64..."
	zip -j $(MACOS_X86_64) target/release/otto LICENSE README.md
	zip -r $(MACOS_X86_64) examples
	@echo "[release] Creating package for Linux x86_64..."
	tar -cvzf $(LINUX_X86_64) \
		-C $$PWD/target/x86_64-unknown-linux-gnu/release otto \
		-C $$PWD LICENSE README.md examples
	@echo "[release] Creating package for Linux armv7..."
	tar -cvzf $(LINUX_ARMV7) \
		-C $$PWD/target/armv7-unknown-linux-gnueabihf/release otto \
		-C $$PWD LICENSE README.md examples

.PHONY: docker
docker: debian

.PHONY: debian
debian: | build-debian docker-run

.PHONY: alpine
alpine: | build-alpine docker-run

.PHONY: build-debian
build-debian:
	docker build -t $(APP) -f debian.Dockerfile .

.PHONY: build-alpine
build-alpine:
	docker build -t $(APP) -f alpine.Dockerfile .

.PHONY: docker-run
docker-run:
	docker run -it --rm \
		-p 9999:9999 \
		-v $$PWD/$(APP).toml:/etc/$(APP)/$(APP).toml \
		-v $$PWD/examples/check_ssl_cert.sh:/usr/local/bin/examples/check_ssl_cert.sh \
		$(APP)
