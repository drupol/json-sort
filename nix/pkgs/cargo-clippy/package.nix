{
  lib,
  cargo,
  clippy,
  json-sort,
}:

json-sort.overrideAttrs (oldAttrs: {
  nativeCheckInputs = (oldAttrs.nativeCheckInputs or [ ]) ++ [
    cargo
    clippy
  ];

  checkPhase = ''
    RUSTFLAGS="-Dwarnings" ${lib.getExe cargo} clippy
  '';
})
