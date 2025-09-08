#!/bin/bash
dx build --release --platform web
cp -rf target/dx/webapp/release/web/public dist
