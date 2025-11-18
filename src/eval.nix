# Takes a path to nixpkgs and a path to the json-encoded list of `pkgs/by-name` attributes.
#
# Returns a value containing information on all Nixpkgs attributes which is decoded on the Rust
# side. See ./eval.rs for the meaning of the returned values.
{ attrsPath, nixpkgsPath }:
let
  attrs = builtins.fromJSON (builtins.readFile attrsPath);

  # We need to check whether attributes are defined via callPackage of the same scope or not.
  overlay = final: prev: {

    # Adds information to each attribute about whether it's defined using this scope's `callPackage`
    callPackage = fn: args: addCallPackageReference (prev.callPackage fn args);
  };

  # We can't just replace attribute values with their info in the overlay, because attributes can
  # depend on other attributes, so this would break evaluation.
  addCallPackageReference =
    value:
    if builtins.isAttrs value then
      value // { _callPackage = true; }
    else
      # It's very rare that `callPackage` doesn't return an attribute set, but it can occur.
      # In such a case we can't really return anything sensible that would include the info, so just
      # don't return the value directly and treat it as if it wasn't a `callPackage`.
      value;

  pkgs = import nixpkgsPath {
    # Don't let the user's home directory influence this result.
    config = { };
    overlays = [ overlay ];
    # We check evaluation and `callPackage` only for x86_64-linux.  Not ideal, but hard to fix.
    system = "x86_64-linux";
  };

  # See AttributeInfo in ./eval.rs for the meaning of this.
  attrInfo = name: value: {
    location = builtins.unsafeGetAttrPos name pkgs;
    attribute_variant =
      if !builtins.isAttrs value then
        { NonAttributeSet = null; }
      else
        {
          AttributeSet = {
            is_derivation = pkgs.lib.isDerivation value;
            is_same_scope_call_package = value._callPackage or false;
          };
        };
  };

  # Information on all attributes that are in `pkgs/by-name`.
  byNameAttrs = builtins.listToAttrs (
    map (name: {
      inherit name;
      value.ByName =
        if !pkgs ? ${name} then
          { Missing = null; }
        else
          # Evaluation failures are not allowed, so don't try to catch them.
          { Existing = attrInfo name pkgs.${name}; };
    }) attrs
  );

  # Information on all attributes that exist but are not in `pkgs/by-name`.
  # We need this to enforce `pkgs/by-name` for new packages.
  nonByNameAttrs = builtins.mapAttrs (
    name: value:
    let
      # Packages outside `pkgs/by-name` often fail evaluation, so we need to handle that.
      output = attrInfo name value;
      result = builtins.tryEval (builtins.deepSeq output null);
    in
    {
      NonByName = if result.success then { EvalSuccess = output; } else { EvalFailure = null; };
    }
  ) (builtins.removeAttrs pkgs attrs);

  # All attributes
  attributes = byNameAttrs // nonByNameAttrs;
in
# We output them in the form [ [ <name> <value> ] ]` such that the Rust side doesn't need to sort
# them again to get deterministic behavior. This is good for testing.
map (name: [
  name
  attributes.${name}
]) (builtins.attrNames attributes)
