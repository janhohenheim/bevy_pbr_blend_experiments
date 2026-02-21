#!/usr/bin/env bash
set -euo pipefail

target="$(pwd)/assets/textures"

mkdir -p "$target"

# Optional: create uncompressed KTX2 (if you really need it)
ktx create \
  --format R8G8B8A8_SRGB \
  --assign-tf srgb \
  --layers 2 \
  assets/raw/brick_basecolor.png \
  assets/raw/render_basecolor.png \
  assets/temp/uncompressed_base_color.ktx2

# print
echo "$target/base_color.ktx2"
# Encode to BC7 texture array with mips
kram encode \
  -f bc7 \
  -type 2darray \
  -srgb \
  -zstd 0 \
  -o "$target/base_color.ktx2" \
  -i assets/temp/uncompressed_base_color.ktx2

rm assets/temp/uncompressed_base_color.ktx2
