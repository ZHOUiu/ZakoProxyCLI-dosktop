#!/usr/bin/env bash
# Cross-compiles zakoai-proxy as libzakoai.so for the Android console APK.
#
# The app hosts the relay in-process (Android 10+ forbids exec'ing a binary from
# app storage), so the whole server is linked as a shared library driven by the
# JNI entry point in src/embedded.rs.
set -euo pipefail

: "${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME to the NDK root}"
: "${ANDROID_API:=28}"

HOST_TAG="windows-x86_64"
case "$(uname -s)" in
  Linux*) HOST_TAG="linux-x86_64" ;;
  Darwin*) HOST_TAG="darwin-x86_64" ;;
esac

TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"
ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT="${JNILIBS_OUT:-$ROOT/target/jniLibs}"

build() {
  local triple="$1" prefix="$2" abi="$3"
  local linker="$TOOLCHAIN/${prefix}${ANDROID_API}-clang"
  [[ "$HOST_TAG" == windows-* ]] && linker="$linker.cmd"
  local tri="${triple//-/_}"
  echo "==> $triple -> $abi"
  env "CARGO_TARGET_${tri^^}_LINKER=$linker" \
    "CC_${tri}=$linker" \
    "AR_${tri}=$TOOLCHAIN/llvm-ar.exe" \
    RUSTFLAGS="-C linker=$linker" \
    cargo +nightly build --release \
      --target "$triple" \
      --lib \
      -Z build-std=std,panic_abort
  mkdir -p "$OUT/$abi"
  cp "target/$triple/release/libzakoai_proxy.so" "$OUT/$abi/libzakoai.so"
  echo "    $(wc -c < "$OUT/$abi/libzakoai.so") bytes"
}

build aarch64-linux-android aarch64-linux-android arm64-v8a
build armv7-linux-androideabi armv7a-linux-androideabi armeabi-v7a
build x86_64-linux-android x86_64-linux-android x86_64
echo "done: $OUT"
