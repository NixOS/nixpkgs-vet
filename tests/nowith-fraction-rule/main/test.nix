{
  config,
  lib,
  pkgs,
}:

let
  cfg = config.services.example;
in
{
  options.services.example = with lib; {
    enable = mkEnableOption "example service";

    package = mkPackageOption pkgs "example" { };

    settings = mkOption {
      type = types.attrsOf types.str;
      default = { };
      description = "Configuration for example.";
    };

    user = mkOption {
      type = types.str;
      default = "example";
      description = "User account under which example runs.";
    };

    group = mkOption {
      type = types.str;
      default = "example";
      description = "Group under which example runs.";
    };
  };
}
