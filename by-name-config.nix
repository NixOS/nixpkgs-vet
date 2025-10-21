# Configure by-name directories in this file.
# Then build with `nix-build -A build` to automatically generate by-name-config-generated.json
# (Or, to generate manually, `nix-instantiate --eval --json --strict by-name-config.nix > by-name-config-generated.json`)
# The following field in the attrsets that make up by_name_dirs are optional: aliases_path.
{
  by_name_dirs = [
    # {
    #   path = "pkgs/development/python-modules/by-name";
    #   attr_path_regex = "^(python3\\d*Packages|python3\\d*.pkgs)\\..*";
    #   unversioned_attr_prefix = "python3Packages";
    #   all_packages_path = "/pkgs/top-level/python-packages.nix";
    #   aliases_path = "/pkgs/top-level/python-aliases.nix";
    # }
    {
      path = "pkgs/development/tcl-modules/by-name";
      attr_path_regex = "^(tcl\\d*Packages)\\..*";
      unversioned_attr_prefix = "tclPackages";
      all_packages_path = "/pkgs/top-level/tcl-packages.nix";
    }
    {
      path = "pkgs/by-name";
      attr_path_regex = ".*"; # There must be exactly one wildcard. All non-wildcard regexes must be mutually exclusive.
      unversioned_attr_prefix = ""; # Ditto for this field.
      all_packages_path = "/pkgs/top-level/all-packages.nix";
      aliases_path = "/pkgs/top-level/aliases.nix";
    }
  ];
}
