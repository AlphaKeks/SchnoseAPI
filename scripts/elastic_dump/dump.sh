#!/usr/bin/env bash

./node_modules/elasticdump/bin/elasticdump \
	--input=$(cat ./credentials.txt) \
	--output=./dump.json \
	--type=data \
	--limit=10000

# vim:filetype=bash
