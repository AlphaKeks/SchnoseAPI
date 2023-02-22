#!/usr/bin/env bash

./node_modules/elasticdump/bin/elasticdump \
	--input=$(cat ./credentials.txt) \
	--output=$1 \
	--type=data \
	--limit=10000

# vim:filetype=bash
