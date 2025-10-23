# Takes a path to nixpkgs and a path to the json-encoded list of `pkgs/by-name` attributes.
#
# Returns a value containing information on all Nixpkgs attributes which is decoded on the Rust
# side. See ./eval.rs for the meaning of the returned values.
{
  attrsPath,
  nixpkgsPath,
  configPath,
}:
let
  # attrs = builtins.fromJSON (builtins.readFile attrsPath);
  rawAttrs = builtins.trace "rawAttrs = ${(builtins.toJSON (builtins.fromJSON (builtins.readFile attrsPath)))}" (builtins.fromJSON (builtins.readFile attrsPath));
  # an attrset where the key is the ID field from by-name-config.nix and the value is a list of attr paths.
  attrsByDir = builtins.trace "attrsByDir = ${(builtins.toJSON (builtins.groupBy (a: a.by_name_dir_id) rawAttrs))}" (builtins.groupBy (a: a.by_name_dir_id) rawAttrs);
  allAttrPaths = map (a: a.attr_path) rawAttrs;
  byNameConfig = builtins.fromJSON (builtins.readFile configPath);
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
        # builtins.trace "eval.nix:36: file = ${file}" file
        file
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
    // { inherit byNameConfig; }
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
      # location = builtins.unsafeGetAttrPos (builtins.trace "eval.nix:84: pname = ${pname}" pname) parent;
      location = builtins.unsafeGetAttrPos pname parent;
      attribute_variant = (
        if !builtins.isAttrs value then
          { NonAttributeSet = null; }
        else
          {
            AttributeSet = {
              is_derivation = pkgs.lib.isDerivation value;
              definition_variant =
                if !(value ? "_callPackageVariant") then
                  { ManualDefinition.is_semantic_call_package = false; }
                else
                  value._callPackageVariant;
            };
          }
      );
    };

  # Information on all attributes that are in a `by-name` directory.
  byNameAttrsForDir = byNameDir: pkgs.lib.mergeAttrsList (
    map (
      package:
      let
        # attrPath = builtins.trace "eval.nix:106: name = ${name}" (pkgs.lib.splitString "." name);
        attrPath = pkgs.lib.splitString "." package.attr_path;
        result = pkgs.lib.setAttrByPath attrPath {
          ByName =
            if !(pkgs.lib.hasAttrByPath attrPath pkgs) then
              { Missing = null; }
            else
              # Evaluation failures are not allowed, so don't try to catch them.
              {
                Existing = attrInfo package.package_name (pkgs.lib.getAttrFromPath attrPath pkgs);
              };
        };
      in
      result
    ) attrsByDir.${byNameDir.id}
  );

  byNameAttrs = pkgs.lib.mergeAttrsList (map byNameAttrsForDir (builtins.filter (dir: builtins.hasAttr dir.id attrsByDir) byNameConfig.by_name_dirs));

  attrSetIsOrContainsDerivation =
    name: value:
    if (!((builtins.tryEval value).success) || !(builtins.isAttrs value)) then
      false # (builtins.trace "attrSetIsOrContainsDerivation: returning false for ${name}" false)
    else
      (
        if pkgs.lib.isDerivation value then
          true
        else if
          (
            (value ? "recurseForDerivations")
            && (builtins.isBool value.recurseForDerivations)
            && (builtins.trace "evaluating value.recurseForDerivations for ${name}" (
              builtins.trace "it has type ${builtins.typeOf (builtins.deepSeq value.recurseForDerivations value.recurseForDerivations)}" value.recurseForDerivations
            ))
          )
        #  then (builtins.any pkgs.lib.id (pkgs.lib.mapAttrsToList attrSetIsOrContainsDerivation value))
        then
          (builtins.trace "${name} has recurseForDerivations true" (
            builtins.any pkgs.lib.id (
              pkgs.lib.mapAttrsToList (
                k: v:
                (builtins.trace "Seeing if ${k} is or contains a derivation" (attrSetIsOrContainsDerivation k v))
              ) value
            )
          ))
        else
          false
      );
  #  then (builtins.trace "attrSetIsOrContainsDerivation: returning true for ${name}" true)
  #  else builtins.trace "attrSetIsOrContainsDerivation: recursing into ${name}" (let result = (builtins.any pkgs.lib.id (pkgs.lib.mapAttrsToList attrSetIsOrContainsDerivation value)); in builtins.seq result (builtins.trace "result of recursing into ${name} reached" result)));
  #  else builtins.any (x: x) (pkgs.lib.mapAttrsToListRecursiveCond (k: v: !(attrSetIsOrContainsDerivation k v)) attrSetIsOrContainsDerivation value);

  markNonByNameAttribute =
    name: value:
    let
      # Packages outside `by-name` directories often fail evaluation, so we need to handle that.
      output = attrInfo name value;
      result = builtins.tryEval (builtins.deepSeq output null);
    in
    if
      (
        result.success
        && (builtins.isAttrs value)
        && !((value ? "type") && (value.type == "derivation"))
        && !(builtins.elem name [
          "buildPackages"
          "targetPackages"
          "__splicedPackages"
          "_callPackageVariant"
          "lib"
        ])
        && !(pkgs.lib.hasPrefix "pkgs" name) # pkgsBuildBuild and friends cause infinite recursion
        && (attrSetIsOrContainsDerivation (builtins.trace "151 name=${name}" name) value)
      )
    # (builtins.tryEval ((pkgs.lib.collect (x: (pkgs.lib.isDerivation x) || (x ? "passthru")) value) != [])
    # (let evalResult = (builtins.tryEval (builtins.deepSeq (pkgs.lib.collect (x: (x ? "passthru") || (pkgs.lib.isDerivation x)) value) (pkgs.lib.collect (x: (x ? "passthru") || (pkgs.lib.isDerivation x)) value)));
    # in (evalResult.success && (evalResult.value != [])))
    then
      (
        let
          recursiveResult = (builtins.mapAttrs markNonByNameAttribute value);
        in
        (builtins.trace "recursing into name = ${name}" (
          builtins.seq (builtins.trace "result of recursing into ${name}: ${builtins.toJSON recursiveResult}" recursiveResult) (
            builtins.trace "done recursing into ${name}" recursiveResult
          )
        ))
      )
    else if result.success then
      {
        NonByName = {
          EvalSuccess = output;
        };
      }
    else
      {
        NonByName = {
          EvalFailure = null;
        };
      };

  # Information on all attributes that exist but are not in a `by-name` directory.
  # We need this to enforce placement in a `by-name` directory for new packages.
  # nonByNameAttrs = pkgs.lib.mapAttrsRecursiveCond (as: !(as ? "_internalCallByNamePackageFile") || !(as ? "type" && as.type == "derivation")) (
  # nonByNameAttrs = let x = pkgs.lib.mapAttrsRecursiveCond (as: !(builtins.trace "trying (builtins.tryEval as).success" (builtins.tryEval as).success) || !(as ? "type" && as.type == "derivation")) (

  nonByNameAttrs = (
    builtins.mapAttrs markNonByNameAttribute (
      builtins.removeAttrs pkgs (
        allAttrPaths
        ++ [
          "lib" # Need to exclude lib to avoid infinite recursion
          # "buildPackages"
          # "targetPackages"
          # "__splicedPackages"
        ]
      )
    )
  );

  # nonByNameAttrs = pkgs.lib.mapAttrsRecursiveCond (as: !(as ? "type" && as.type == "derivation")) (
  #   name: value:
  #   let
  #     # Packages outside  `by-name` directories often fail evaluation, so we need to handle that.
  #     output = attrInfo name value;
  #     result = builtins.tryEval (builtins.deepSeq output null);
  #   in
  #   {
  #     NonByName = if result.success then { EvalSuccess = output; } else { EvalFailure = null; };
  #   }
  # ) (builtins.removeAttrs pkgs (attrs ++ [ "lib" ])); # Need to exclude lib to avoid infinite recursion

  # All attributes
  attributes = byNameAttrs // nonByNameAttrs;
  result =
    pkgs.lib.mapAttrsToListRecursiveCond
      (attrPath: attrValue: !((attrValue ? "NonByName") || (attrValue ? "ByName")))
      (attrPath: attrValue: [
        attrPath
        attrValue
      ])
      attributes;
in
# We output them in the form [ [ <name> <value> ] ]` such that the Rust side doesn't need to sort
# them again to get deterministic behavior. This is good for testing.
# result
result
# attributes
# pkgs.lib.traceSeq "eval.nix:154: result'' = ${toStringRec result''}" result'' # (throw "")
