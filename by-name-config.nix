# Configure by-name directories in this file.
# Then build with `nix-build -A build` to automatically generate by-name-config-generated.json
# (Or, to generate manually, `nix-instantiate --eval --json --strict by-name-config.nix > by-name-config-generated.json`)
{
  by_name_dirs = [
    {
      path = "pkgs/development/python-modules/by-name";
      attr_path_regex = "^(python3\\d*Packages|python3\\d*.pkgs)\\..*";
      unversioned_attr_prefix = "python3Packages";
    }
    {
      path = "pkgs/development/tcl-modules/by-name";
      attr_path_regex = "^(tcl\\d*Packages)\\..*";
      unversioned_attr_prefix = "tclPackages";
    }
    {
      path = "pkgs/by-name";
      attr_path_regex = ".*"; # There must be exactly one wildcard. All non-wildcard regexes must be mutually exclusive.
      unversioned_attr_prefix = ""; # Ditto for this field.
    }
  ];
}