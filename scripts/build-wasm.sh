#!/bin/bash
set -e

# WASM Build Script for Chess Engine
# Usage:
#   ./scripts/build-wasm.sh          # Build release version
#   ./scripts/build-wasm.sh --dev    # Build development version
#   ./scripts/build-wasm.sh --release # Build release version (explicit)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_CRATE_DIR="$PROJECT_ROOT/crates/engine-bridge-wasm"
OUTPUT_DIR="$PROJECT_ROOT/apps/web/public/wasm"

# Default to release build
BUILD_TYPE="release"
WASM_PACK_ARGS="--release"

# Parse arguments
for arg in "$@"; do
  case $arg in
    --dev)
      BUILD_TYPE="dev"
      WASM_PACK_ARGS="--dev"
      shift
      ;;
    --release)
      BUILD_TYPE="release"
      WASM_PACK_ARGS="--release"
      shift
      ;;
    *)
      echo "Unknown argument: $arg"
      echo "Usage: $0 [--dev|--release]"
      exit 1
      ;;
  esac
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Building WASM Engine ($BUILD_TYPE mode)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found!"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

echo "✓ wasm-pack found"

# Create output directory
mkdir -p "$OUTPUT_DIR"
echo "✓ Output directory: $OUTPUT_DIR"

# Build with wasm-pack
echo ""
echo "⚙️  Running wasm-pack build..."
cd "$WASM_CRATE_DIR"

wasm-pack build \
  --target web \
  --out-dir "$OUTPUT_DIR" \
  $WASM_PACK_ARGS

echo "✓ wasm-pack build complete"

# Check if wasm-opt is available (optional for dev builds)
if command -v wasm-opt &> /dev/null; then
  if [ "$BUILD_TYPE" = "release" ]; then
    echo ""
    echo "⚙️  Running wasm-opt for additional size optimization..."

    WASM_FILE="$OUTPUT_DIR/engine_bridge_wasm_bg.wasm"
    if [ -f "$WASM_FILE" ]; then
      # Backup original
      cp "$WASM_FILE" "$WASM_FILE.bak"

      # Run wasm-opt with aggressive size optimization
      # Note: These flags should match package.metadata.wasm-pack.profile.release in Cargo.toml
      wasm-opt -Oz \
        --enable-mutable-globals \
        --enable-bulk-memory \
        --enable-nontrapping-float-to-int \
        --enable-sign-ext \
        --strip-debug \
        --strip-producers \
        -o "$WASM_FILE" \
        "$WASM_FILE.bak"

      # Remove backup
      rm "$WASM_FILE.bak"

      echo "✓ wasm-opt optimization complete"
    else
      echo "⚠️  WASM file not found at expected location: $WASM_FILE"
    fi
  else
    echo "⏭️  Skipping wasm-opt for dev build"
  fi
else
  echo "⚠️  wasm-opt not found (optional)"
  echo "   Install binaryen for additional size optimization:"
  echo "   - macOS: brew install binaryen"
  echo "   - Linux: apt install binaryen"
fi

# Display build stats
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Build Statistics"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WASM_FILE="$OUTPUT_DIR/engine_bridge_wasm_bg.wasm"
if [ -f "$WASM_FILE" ]; then
  WASM_SIZE=$(du -h "$WASM_FILE" | cut -f1)
  WASM_SIZE_BYTES=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
  WASM_SIZE_KB=$((WASM_SIZE_BYTES / 1024))

  echo "WASM binary:     $WASM_SIZE ($WASM_SIZE_KB KB)"

  # Check if gzip is available
  if command -v gzip &> /dev/null; then
    GZIP_SIZE=$(gzip -c "$WASM_FILE" | wc -c)
    GZIP_SIZE_KB=$((GZIP_SIZE / 1024))
    echo "WASM (gzipped):  $GZIP_SIZE_KB KB"

    if [ "$BUILD_TYPE" = "release" ]; then
      if [ $GZIP_SIZE_KB -lt 1024 ]; then
        echo "✓ Excellent! Gzipped size under 1MB"
      elif [ $GZIP_SIZE_KB -lt 2560 ]; then
        echo "✓ Good! Gzipped size under 2.5MB"
      else
        echo "⚠️  Warning: Gzipped size exceeds 2.5MB target"
      fi
    fi
  fi
else
  echo "⚠️  WASM file not found"
fi

echo ""
echo "Output files:"
ls -lh "$OUTPUT_DIR"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ Build Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
