#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
image='openapitools/openapi-generator-cli:v7.15.0'
properties='apis,models,supportingFiles,apiDocs=false,modelDocs=false,apiTests=false,modelTests=false'

for language in kotlin swift5 rust csharp; do
  case "$language" in
    kotlin) name=kotlin; options='packageName=org.mensetsu.contracts,library=jvm-okhttp4' ;;
    swift5) name=swift; options='projectName=MensetsuContracts' ;;
    rust) name=rust; options='packageName=mensetsu_contracts' ;;
    csharp) name=csharp; options='packageName=Mensetsu.Contracts' ;;
  esac
  docker run --rm --user "$(id -u):$(id -g)" \
    -v "$root:/work:ro" -v "$tmp:/output" "$image" generate \
    -i /work/contracts/rest/openapi.json -g "$language" -o "/output/$name" \
    --additional-properties="$options" \
    --global-property="$properties"
done

mkdir -p "$tmp/native/kotlin" "$tmp/native/swift" "$tmp/native/rust" "$tmp/native/csharp/src/Mensetsu.Contracts"
cp -R "$tmp/kotlin/src" "$tmp/native/kotlin/"
cp -R "$tmp/swift/MensetsuContracts" "$tmp/swift/Package.swift" "$tmp/native/swift/"
cp -R "$tmp/rust/src" "$tmp/rust/Cargo.toml" "$tmp/native/rust/"
cp -R "$tmp/csharp/src/Mensetsu.Contracts/Api" "$tmp/csharp/src/Mensetsu.Contracts/Model" \
  "$tmp/csharp/src/Mensetsu.Contracts/Client" "$tmp/csharp/src/Mensetsu.Contracts/Extensions" \
  "$tmp/csharp/src/Mensetsu.Contracts/Mensetsu.Contracts.csproj" "$tmp/native/csharp/src/Mensetsu.Contracts/"

if [[ "${1:-}" == '--check' ]]; then
  diff -ru --exclude=bin --exclude=obj --exclude=target --exclude=Cargo.lock "$tmp/native" "$root/contracts/native"
else
  mkdir -p "$root/contracts"
  rm -rf "$root/contracts/native"
  cp -R "$tmp/native" "$root/contracts/native"
fi
