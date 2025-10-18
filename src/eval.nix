# Takes a path to nixpkgs and a path to the json-encoded list of `pkgs/by-name` attributes.
#
# Returns a value containing information on all Nixpkgs attributes which is decoded on the Rust
# side. See ./eval.rs for the meaning of the returned values.
{
  attrsPath,
  nixpkgsPath,
  configPath ? null,
}:
let
  attrs = builtins.fromJSON (builtins.readFile attrsPath);

  # We need to check whether attributes are defined manually e.g. in `all-packages.nix`,
  # automatically by the `pkgs/by-name` overlay, or neither. The only way to do so is to override
  # `callPackage` and `_internalCallByNamePackageFile` with our own version that adds this
  # information to the result, and then try to access it.
  overlay = final: prev: {

    # Adds information to each attribute about whether it's manually defined using `callPackage`
    callPackage =
      fn: args:
      addVariantInfo (prev.callPackage fn args) {
        # This is a manual definition of the attribute, and it's a `1callPackage`, specifically a
        # semantic `callPackage`.
        ManualDefinition.is_semantic_call_package = true;
      };

    # Adds information to each attribute about whether it's automatically defined by the
    # `pkgs/by-name` overlay. This internal attribute is only used by that overlay.
    #
    # This overrides the above `callPackage` information. It's OK because we don't need that one,
    # since `pkgs/by-name` always uses `callPackage` underneath.
    _internalCallByNamePackageFile =
      file:
      addVariantInfo (prev._internalCallByNamePackageFile (
        builtins.trace "file at eval:34: ${file}" file
      )) { AutoDefinition = null; };
  };

  # We can't just replace attribute values with their info in the overlay, because attributes can
  # depend on other attributes, so this would break evaluation.
  addVariantInfo =
    value: variant:
    if builtins.isAttrs value then
      value // { _callPackageVariant = variant; }
    else
      # It's very rare that `callPackage` doesn't return an attribute set, but it can occur.
      # In such a case we can't really return anything sensible that would include the info, so just
      # don't return the value directly and treat it as if it wasn't a `callPackage`.
      value;

  pkgs = import nixpkgsPath (
    {
      # Don't let the user's home directory influence this result.
      config = { };
      overlays = [ overlay ];
      # We check evaluation and `callPackage` only for x86_64-linux.  Not ideal, but hard to fix.
      system = "x86_64-linux";
    }
    // (
      if (configPath != null) then
        {
          byNameConfig = builtins.fromJSON (builtins.readFile configPath);
        }
      else
        { }
    )
  );

  # See AttributeInfo in ./eval.rs for the meaning of this.
  attrInfo =
    attrPath_0: value:
    let
      attrPath = if builtins.isList attrPath_0 then attrPath_0 else [ attrPath_0 ];
      attrPathLength = builtins.length attrPath;
      pname = pkgs.lib.last attrPath;
      parent =
        if attrPathLength == 1 then
          pkgs
        else
          pkgs.lib.attrsets.getAttrFromPath (pkgs.lib.take (attrPathLength - 1) attrPath) pkgs;
    in
    {
      location = builtins.unsafeGetAttrPos (builtins.trace "eval:73: pname is ${pname}" pname) parent;
      attribute_variant =
        if !builtins.isAttrs value then
          { NonAttributeSet = null; }
        else
          {
            AttributeSet = {
              is_derivation = pkgs.lib.isDerivation value;
              definition_variant =
                if !value ? _callPackageVariant then
                  { ManualDefinition.is_semantic_call_package = false; }
                else
                  value._callPackageVariant;
            };
          };
    };

  # Information on all attributes that are in a `by-name` directory.
  byNameAttrs = pkgs.lib.mergeAttrsList (
    map (
      name:
      let
        attrPath = (pkgs.lib.splitString "." name);
        result = pkgs.lib.setAttrByPath attrPath {
          ByName =
            if !(pkgs.lib.hasAttrByPath attrPath pkgs) then
              { Missing = null; }
            else
              # Evaluation failures are not allowed, so don't try to catch them.
              {
                Existing = attrInfo (builtins.trace "name @ line 100: ${name}" name) (
                  pkgs.lib.getAttrFromPath attrPath pkgs
                );
              };
        };
      in
      result
    ) attrs
  );

  # Information on all attributes that exist but are not in a `by-name` directory.
  # We need this to enforce placement in a `by-name` directory for new packages.
  # nonByNameAttrs = pkgs.lib.mapAttrsRecursiveCond (as: !(as ? "_internalCallByNamePackageFile") || !(as ? "type" && as.type == "derivation")) (
  nonByNameAttrs = pkgs.lib.mapAttrsRecursiveCond (as: !(as ? "type" && as.type == "derivation")) (
    name: value:
    let
      # Packages outside  `by-name` directories often fail evaluation, so we need to handle that.
      output = attrInfo name value;
      result = builtins.tryEval (builtins.deepSeq output null);
    in
    {
      NonByName = if result.success then { EvalSuccess = output; } else { EvalFailure = null; };
    }
  ) (builtins.removeAttrs pkgs (attrs ++ [ "lib" ])); # Need to exclude lib to avoid infinite recursion

  # All attributes
  attributes = byNameAttrs // nonByNameAttrs;
  result = builtins.attrValues (
    pkgs.lib.mapAttrsRecursiveCond (as: !(as ? "NonByName" || as ? "ByName")) (attrPath: attrValue: [
      attrPath
      attrValue
    ]) attributes
  );
  # FIXME: this is a gross workaround, need to debug why we get this at the end, unlike all the others
  # {"tclcurl":[["tclPackages","tclcurl"],{"NonByName":{"EvalSuccess":{"attribute_variant":{"AttributeSet":{"definition_variant":{"AutoDefinition":null},"is_derivation":true}},"location":null}}}]}
  result' = map (x: if builtins.isAttrs x then builtins.head (builtins.attrValues x) else x) result;
in
# We output them in the form [ [ <name> <value> ] ]` such that the Rust side doesn't need to sort
# them again to get deterministic behavior. This is good for testing.
builtins.trace (pkgs.lib.deepSeq result' result') result' # (throw "")
