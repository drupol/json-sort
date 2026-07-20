{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];

  perSystem = {
    treefmt = {
      imports = [
        inputs.self.treefmtModules.default
      ];
      projectRootFile = "flake.nix";
      programs = {
        jsonfmt.enable = true;
        json-sort.enable = true;
        nixfmt.enable = true;
        prettier.enable = true;
        statix.enable = true;
        typos.enable = true;
        yamlfmt.enable = true;
      };
      settings = {
        no-cache = true;
        on-unmatched = "warn";
      };
    };
  };
}
