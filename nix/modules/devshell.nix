{
  inputs,
  ...
}:
{
  imports = [
    inputs.make-shell.flakeModules.default
    inputs.git-hooks.flakeModule
  ];

  perSystem =
    { pkgs, ... }:
    {
      pre-commit.settings.hooks = {
        commitizen.enable = true;
      };

      make-shells.default = {
        packages = with pkgs; [
          binaryen # wasm-opt
          cargo
          caddy
          clippy
          hyperfine
          just
          lld
          nodejs
          rust-analyzer
          rustc
          rustfmt
          wasm-pack
          wasm-bindgen-cli
        ];

        shellHook = ''
          export PATH=$PWD/target/debug:$PATH
          export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        '';
      };
    };
}
