{
  lib,
  rustPlatform,
  versionCheckHook,
}:

rustPlatform.buildRustPackage {
  pname = "json-sort";
  version = "1.1.6";

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

  cargoHash = "sha256-0rW+wCA2LdLGlXnHaRIzQ4WK25m4rl39HV46mjfNws8=";

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
