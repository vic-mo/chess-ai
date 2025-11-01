#!/bin/bash
set -e

# Build protocol package
npm install --prefix packages/protocol
npm run build --prefix packages/protocol

# Build web app
cd apps/web
sed -i.bak '/"@chess-ai\/protocol":/d' package.json
npm install --legacy-peer-deps
mkdir -p node_modules/@chess-ai
cp -r ../../packages/protocol node_modules/@chess-ai/
npm run build
