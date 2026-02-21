#!/usr/bin/env bash
set -euo pipefail

# TODO: replace with kram
ktx create \
  --format R8G8B8A8_SRGB \
  --assign-tf srgb \
  --layers 2 \
  assets/textures/raw/brick_basecolor.png \
  assets/textures/raw/render_basecolor.png \
  assets/temp/uncompressed_base_color.ktx2

kram encode \
  -f bc7 \
  -type 2darray \
  -srgb \
  -zstd 0 \
  -o assets/textures/base_color.ktx2 \
  -i assets/temp/uncompressed_base_color.ktx2

rm assets/temp/uncompressed_base_color.ktx2

# TODO: fix this
kram encode -f bc4 -type 2d -srclin -i assets/textures/raw/wear_mask.png -o assets/textures/wear_mask.ktx2

# TODO: from Chris:

# ktx create --format R8G8B8_UNORM --assign-tf linear --layers 3 assets/raw_assets/floor_graph_normal_map.png assets/raw_assets/grass_graph_normal_map.png assets/raw_assets/stone_graph_normal_map.png $'($target)/uncompressed_normal_map.ktx2'
# ~/resources/kram-macos/kram encode -f bc5 -type 2darray -normal -o $'($target)/normal_map.ktx2' -i $'($target)/uncompressed_normal_map.ktx2'

# ktx create --format R8_UNORM --assign-tf linear --layers 3 assets/raw_assets/floor_graph_depth_map.png assets/raw_assets/grass_graph_depth_map.png assets/raw_assets/stone_graph_depth_map.png $'($target)/uncompressed_depth_map.ktx2'
# ~/resources/kram-macos/kram encode -f bc4 -type 2darray -srclin  -o $'($target)/depth_map.ktx2' -i $'($target)/uncompressed_depth_map.ktx2'
