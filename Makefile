APP_NAME=otto

build:
	cargo build --release

run:
	cargo run -- --config $(APP_NAME).toml

lint:
	cargo clippy --workspace --tests --all-features -- -D warnings

docker:
	docker build -t otto .
	docker run -it --rm \
		-p 9999:9999 \
		-v $$PWD/$(APP_NAME).toml:/etc/$(APP_NAME)/$(APP_NAME).toml \
		$(APP_NAME)
