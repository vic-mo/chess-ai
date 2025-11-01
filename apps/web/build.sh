#!/bin/bash
set -e
cd ../..
npm i --prefix packages/protocol
npm run build --prefix packages/protocol
sed -i '' '/@chess-ai/d' apps/web/package.json
npm i --legacy-peer-deps --prefix apps/web
mkdir -p apps/web/node_modules/@chess-ai
cp -r packages/protocol apps/web/node_modules/@chess-ai/
npm run build --prefix apps/web
