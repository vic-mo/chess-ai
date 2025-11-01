#!/bin/bash
set -e
cd "$(dirname "$0")"
npm i --prefix packages/protocol
npm run build --prefix packages/protocol
grep -v '@chess-ai/protocol' apps/web/package.json > apps/web/package.json.tmp
mv apps/web/package.json.tmp apps/web/package.json
npm i --legacy-peer-deps --prefix apps/web
mkdir -p apps/web/node_modules/@chess-ai
cp -r packages/protocol apps/web/node_modules/@chess-ai/
npm run build --prefix apps/web
