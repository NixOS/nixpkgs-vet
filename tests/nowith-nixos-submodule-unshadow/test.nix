{ config
, lib
}:

let
  cfg = config.foo;
in {
  options.foo = lib.mkOption {
    type = # random example from nixpkgs
      lib.types.attrsOf (
        lib.types.either lib.types.path (lib.types.submodule {
          options = {
            service = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "The service on which to perform \<action\> after fetching.";
            };

            action = lib.mkOption {
              type = lib.types.addCheck lib.types.str (
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

