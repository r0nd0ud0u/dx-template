#!/bin/bash
dx bundle --platform web --release --out-dir bundle
cp .env_template bundle/.env_template
cp db.sqlite bundle/db.sqlite
