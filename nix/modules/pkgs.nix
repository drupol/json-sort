{ inputs, withSystem, ... }:
{
  imports = [
    inputs.pkgs-by-name-for-flake-parts.flakeModule
  ];

  perSystem =
    { config, ... }:
    {
      pkgsDirectory = ../pkgs;
      packages.default = config.packages.json-sort;
    };

  flake = {
    overlays.default =
      final: prev:
      withSystem prev.stdenv.hostPlatform.system (
        { config, ... }:
        {
          inherit (config.packages) json-sort;
        }
      );
  };
}
