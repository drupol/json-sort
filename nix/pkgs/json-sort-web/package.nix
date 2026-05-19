{
  lib,
  json-sort,
  rustPlatform,
  cargo,
  rustc,
  wasm-bindgen-cli,
  writableTmpDirAsHomeHook,
  binaryen,
  lld,
  ...
}:

json-sort.overrideAttrs (oldAttrs: {
  pname = "${json-sort.pname}-web";

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../Cargo.toml
      ../../../Cargo.lock
      ../../../src
      ../../../web
    ];
  };

  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
    cargo
    rustc
    rustPlatform.cargoSetupHook
    wasm-bindgen-cli
    writableTmpDirAsHomeHook
    binaryen
    lld
  ];

  cargoBuildFlags = [
    "--target"
    "wasm32-unknown-unknown"
    "--lib"
  ];

  postBuild = ''
    wasm-bindgen \
      --target web \
      --out-dir web/pkg \
      target/wasm32-unknown-unknown/release/json_sort.wasm
    wasm-opt -O --enable-bulk-memory \
      -o web/pkg/json_sort_bg.wasm \
      web/pkg/json_sort_bg.wasm
  '';

  installPhase = ''
    runHook preInstall

    cp -r web $out

    runHook postInstall
  '';

  doInstallCheck = false;
})
