{
  inputs,
  ...
}:
let
  treefmtModule =
    {
      lib,
      config,
      pkgs,
      ...
    }:
    let
      cfg = config.programs.json-sort;
    in
    {
      config = lib.mkIf cfg.enable {
        settings.formatter.json-sort = {
          inherit (cfg) includes;
          command = lib.getExe cfg.package;
          options = [ "--fix" ];
        }
        // lib.optionalAttrs (cfg.excludes != [ ]) { inherit (cfg) excludes; }
        // lib.optionalAttrs (cfg.priority != null) { inherit (cfg) priority; };
      };
      options.programs.json-sort = {
        enable = lib.mkEnableOption "json-sort, the JSON formatter";
        package = lib.mkOption {
          default = pkgs.json-sort;
          defaultText = lib.literalExpression "pedantix.packages.\${system}.pedantix-wrapped";
          description = "The pedantix package to run. The default wrapper ships nixfmt, alejandra and nixpkgs-fmt on PATH; use the unwrapped `pedantix` package if you provide the base formatter yourself.";
          type = lib.types.package;
        };
        excludes = lib.mkOption {
          default = [ ];
          description = "Path / file patterns to exclude.";
          example = [ "generated/*.json" ];
          type = lib.types.listOf lib.types.str;
        };
        includes = lib.mkOption {
          default = [ "*.json" ];
          description = "Path / file patterns to include.";
          type = lib.types.listOf lib.types.str;
        };
        priority = lib.mkOption {
          default = null;
          description = "treefmt priority, for ordering relative to other formatters on the same files.";
          type = lib.types.nullOr lib.types.int;
        };
      };
    };
in
{
  imports = [
    inputs.flake-parts.flakeModules.flakeModules
  ];

  flake = {
    flakeModules = {
      default =
        { flake-parts-lib, ... }:
        {
          options.perSystem = flake-parts-lib.mkPerSystemOption {
            config.treefmt = {
              imports = [ treefmtModule ];
            };
          };
        };
    };
    treefmtModules = {
      default = treefmtModule;
    };
  };
}
