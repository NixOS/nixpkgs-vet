{
  config,
  lib,
}:

let
  cfg = config.foo;

  inherit (lib)
    mkOption
    elem
    ;

  inherit (lib.types)
    attrsOf
    either
    path
    submodule
    nullOr
    str
    addCheck
    ;
in
{
  options.foo = mkOption {
    type = # random example from nixpkgs
      attrsOf (
        either path (submodule {
          options = {
            service = mkOption {
              type = nullOr str;
              default = null;
              description = "The service on which to perform \<action\> after fetching.";
            };

            action = mkOption {
              type = addCheck str (
                x:
                cfg.svcManager == "command"
                || elem x [
                  "restart"
                  "reload"
                  "nop"
                ]
              );
              default = "nop";
              description = "The action to take after fetching.";
            };
          };
        })
      );
  };
}
