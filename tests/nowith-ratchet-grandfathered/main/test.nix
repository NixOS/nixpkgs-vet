{
  config,
  lib,
}:

let
  cfg = config.foo;
in
{
  options.bar = lib.mkOption {
    type = lib.types.str;
  };
  options.foo = lib.mkOption {
    type = # random example from nixpkgs
      with lib.types;
      attrsOf (
        either path (submodule {
          options = {
            service = lib.mkOption {
              type = nullOr str;
              default = null;
              description = "The service on which to perform \<action\> after fetching.";
            };

            action = lib.mkOption {
              type = addCheck str (
                x:
                cfg.svcManager == "command"
                || lib.elem x [
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
