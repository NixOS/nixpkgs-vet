# Takes a path to nixpkgs and a path to the json-encoded list of `pkgs/by-name` attributes.
#
# Returns a value containing information on all Nixpkgs attributes which is decoded on the Rust
# side. See ./eval.rs for the meaning of the returned values.
{ attrsPath, nixpkgsPath }:
let
  attrs = builtins.fromJSON (builtins.readFile attrsPath);

  # Copied from lib
  attrByPath =
    attrPath: default: set:
    let
      lenAttrPath = builtins.length attrPath;
      attrByPath' =
        n: s:
        (
          if n == lenAttrPath then
            s
          else
            (
              let
                attr = builtins.elemAt attrPath n;
              in
              if s ? ${attr} then attrByPath' (n + 1) s.${attr} else default
            )
        );
    in
    attrByPath' 0 set;
  sublist =
    start: count: list:
    let
      len = builtins.length list;
    in
    builtins.genList (n: builtins.elemAt list (n + start)) (
      if start >= len then
        0
      else if start + count > len then
        len - start
      else
        count
    );

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
    # TODO: Check for all supported systems to ensure that packages don't fail evaluation on other systems
    system = "x86_64-linux";
  };

  # See AttributeInfo in ./eval.rs for the meaning of this.
  attrInfo = path: value: {
    location = builtins.unsafeGetAttrPos (builtins.elemAt path (builtins.length path - 1)) (
      attrByPath (sublist 0 (builtins.length path - 1) path) (throw "No such attr") pkgs
    );
    attribute_variant =
      if !builtins.isAttrs value then
        { NonAttributeSet = null; }
      else
        let
          # Not all attrs (or even all derivations) have an `overrideAttrs`,
          # only those constructed by something based on `stdenv.mkDerivation`
          overrideValue = value.overrideAttrs or (_: value);

          # Disallows people getting around actually setting `strictDeps`
          # and `__structuredAttrs` by doing something like:
          # {
          #   passthru.strictDeps = true;
          # }
          # or
          # package = package-final // {
          #   __structuredAttrs = true;
          # };
          cleanPackage = overrideValue (
            _: prev: {
              passthru = removeAttrs (prev.passthru or { }) [
                "__structuredAttrs"
                "strictDeps"
              ];
            }
          );
        in
        {
          AttributeSet = {
            is_derivation = pkgs.lib.isDerivation value;
            missing_maintainers = value.meta.missingMaintainers or null;
            has_no_maintainers_but_dependents = value.meta.hasNoMaintainersButDependents or false;
            strict_deps = cleanPackage.strictDeps or false;
            structured_attrs = cleanPackage.__structuredAttrs or false;
            meta_position = value.meta.position or null;
            pname = value.pname or null;
            name = value.name or null;
            is_same_scope_call_package = value._callPackage or false;
          };
        };
  };

  # Information on all attributes that are in `pkgs/by-name`.
  byNameAttrs = map (name: [
    [ name ]
    {
      ByName =
        if !pkgs ? ${name} then
          { Missing = null; }
        else
          # Evaluation failures are not allowed, so don't try to catch them.
          { Existing = attrInfo [ name ] pkgs.${name}; };
    }
  ]) attrs;

  ciEvalAttrpaths = nixpkgsPath + "/ci/eval/attrpaths.nix";

  allAttrs = (import ciEvalAttrpaths { }).paths;
  nonByNameAttrsList =
    builtins.filter (p: !(builtins.length p > 0 && builtins.elemAt p 0 == "tests"))
      (
        if builtins.pathExists ciEvalAttrpaths then

          builtins.filter (
            path: !(builtins.length path == 1 && builtins.elem (builtins.elemAt path 0) attrs)
          ) allAttrs
        else
          map (a: [ a ]) (builtins.attrNames (builtins.removeAttrs pkgs attrs))
      );

  # Information on all attributes that exist but are not in `pkgs/by-name`.
  # We need this to enforce `pkgs/by-name` for new packages.
  nonByNameAttrs = builtins.map (
    path:
    let
      value = attrByPath path (throw "No ${toString path}") pkgs;
      # Packages outside `pkgs/by-name` often fail evaluation, so we need to handle that.
      output = attrInfo path value;
      result = builtins.tryEval (builtins.deepSeq output null);
    in
    [
      path
      {
        NonByName = if result.success then { EvalSuccess = output; } else { EvalFailure = null; };
      }
    ]
  ) nonByNameAttrsList;
in
# We output them in the form [ [ <attrpath> <value> ] ]` such that the Rust side doesn't need to sort
# them again to get deterministic behavior. This is good for testing.
byNameAttrs ++ nonByNameAttrs
