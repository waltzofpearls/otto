APP_NAME=otto

build:
	cargo build --release

run:
	cargo run -- --config $(APP_NAME).toml

lint:
	cargo clippy --all-targets -- -D warnings

docker:
	docker build -t otto .
	docker run -it --rm -v $$PWD/$(APP_NAME).toml:/etc/$(APP_NAME)/$(APP_NAME).toml $(APP_NAME)
