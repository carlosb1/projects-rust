#!/bin/bash
set -e

dx build --release --platform web

DIST="../app/dist"
BUILD="target/dx/webapp/release/web/public"

rm -rf "$DIST"
mkdir -p "$DIST"

cp -rf "$BUILD"/. "$DIST"/