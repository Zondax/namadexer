#!/bin/sh

NAMADA_VERSION := 0.31.1
BASE_URL := https://raw.githubusercontent.com/anoma/namada
URL := $(BASE_URL)/v$(NAMADA_VERSION)/wasm/checksums.json

CHECK_CURL := $(shell command -v curl 2> /dev/null)
CHECK_WGET := $(shell command -v wget 2> /dev/null)

ifdef CHECK_CURL
DOWNLOAD_CMD = curl -L -o
else ifdef CHECK_WGET
DOWNLOAD_CMD = wget -O
else
$(error Neither curl nor wget are available on your system)
endif

# Determine the OS and set the appropriate value for OS
UNAME := $(shell uname)
ifeq ($(UNAME),Linux)
    OS := linux
endif
ifeq ($(UNAME),Darwin)
    OS := osx
endif

# Set a default value for OS if it's not recognized
OS ?= linux

download-checksum:
	@if [ ! -f checksums.json ]; then \
		echo $(URL); \
		$(DOWNLOAD_CMD) checksums.json $(URL); \
	fi

install-deps:
	# Use OS variable in the download URL and unzip command
	$(DOWNLOAD_CMD) protoc-3.16.3-$(OS)-x86_64.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.16.3/protoc-3.16.3-$(OS)-x86_64.zip
	unzip protoc-3.16.3-$(OS)-x86_64.zip -d ./protoc


postgres:
	docker run --name postgres -e POSTGRES_PASSWORD=wow -e POSTGRES_DB=blockchain -p 5432:5432 -d postgres:14

build: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" cargo build --features prometheus

run_indexer: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" cargo r --bin indexer

run_server: download-checksum
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" cargo r --bin server

benchmarks: download-checksum 
	INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" PATH="${PWD}/protoc/bin:${PATH}" cargo bench 

compose:
	docker compose -f contrib/docker-compose.yaml up

test: download-checksum
	cargo test save_block -- --nocapture
	cargo test block_tests -- --nocapture
