#!/bin/sh

NAMADA_VERSION := 0.21.1
BASE_URL := https://raw.githubusercontent.com/anoma/namada
URL := $(BASE_URL)/v$(NAMADA_VERSION)/wasm/checksums.json

CHECK_CURL := $(shell command -v curl 2> /dev/null)
CHECK_WGET := $(shell command -v wget 2> /dev/null)

ifdef CHECK_CURL
DOWNLOAD_CMD = curl -o
else ifdef CHECK_WGET
DOWNLOAD_CMD = wget -O
else
$(error Neither curl nor wget are available on your system)
endif

download-checksum:
	@if [ ! -f checksums.json ]; then \
		echo $(URL); \
		$(DOWNLOAD_CMD) checksums.json $(URL); \
	fi

install-deps:
	# We need a specific protoc version
	$(DOWNLOAD_CMD) https://github.com/protocolbuffers/protobuf/releases/download/v3.16.3/protoc-3.16.3-linux-x86_64.zip
	unzip protoc-3.16.3-linux-x86_64.zip -d ./protoc

postgres:
	docker run --name postgres -e POSTGRES_PASSWORD=wow -e POSTGRES_DB=blockchain -p 5432:5432 -d postgres

build: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" cargo build --features prometheus

run_indexer: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" RUST_LOG="namada_prototype=info" cargo r --release --bin indexer --features prometheus

run_server: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" RUST_LOG="namada_prototype=info" cargo r --release  --bin server --features prometheus

compose:
	docker compose -f contrib/docker-compose.yaml up

test: download-checksum
	cargo test -- --nocapture
