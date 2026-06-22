{
  lib,
  rustPlatform,
  versionCheckHook,
}:

rustPlatform.buildRustPackage {
  pname = "json-sort";
  version = "1.1.5";

  __structuredAttrs = true;

  src = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../Cargo.toml
      ../../../Cargo.lock
      ../../../tests
      ../../../src
    ];
  };

  cargoHash = "sha256-HWYsZjPeIG7OuboAZeF4CxmfGVbqX0q9I8OCM4ilj+E=";

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  meta = {
    description = "Command-line tool to sort JSON object keys in-place, preserving formatting and comments";
    homepage = "https://github.com/drupol/json-sort";
    license = lib.licenses.eupl12;
    mainProgram = "json-sort";
    maintainers = with lib.maintainers; [ drupol ];
    platforms = lib.platforms.all;
  };
}
