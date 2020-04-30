#!/bin/bash
set -ex

MEILISEARCH_ADDRESS="127.0.0.1:7700"

curl -i -X DELETE "$MEILISEARCH_ADDRESS/indexes/news" 
curl -i -X POST "$MEILISEARCH_ADDRESS/indexes" --data "{ \"name\": \"News\", \"uid\":\"news\", \"primaryKey\":\"id\" }"

