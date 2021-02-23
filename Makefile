APP_NAME=otto

.PHONY: build
build:
	cargo build --release

.PHONY: run
run:
	cargo run -- --config $(APP_NAME).toml

.PHONY: lint
lint:
	cargo clippy --workspace --tests --all-features -- -D warnings

.PHONY: docker
docker: debian

.PHONY: debian
debian: | build-debian docker-run

.PHONY: alpine
alpine: | build-alpine docker-run

.PHONY: build-debian
build-debian:
	docker build -t $(APP_NAME) -f debian.Dockerfile .

.PHONY: build-alpine
build-alpine:
	docker build -t $(APP_NAME) -f alpine.Dockerfile .

.PHONY: docker-run
docker-run:
	docker run -it --rm \
		-p 9999:9999 \
		-v $$PWD/$(APP_NAME).toml:/etc/$(APP_NAME)/$(APP_NAME).toml \
		-v $$PWD/examples/check_ssl_cert.sh:/usr/local/bin/examples/check_ssl_cert.sh \
		$(APP_NAME)
