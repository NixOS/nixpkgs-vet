# Configure by-name directories in this file.
# Then build with `nix-build -A build` to automatically generate by-name-config-generated.json
# (Or, to generate manually, `nix-instantiate --eval --json --strict by-name-config.nix > by-name-config-generated.json`)
# In the attrsets that make up by_name_dirs:
#   * The aliases_path field is optional.
#   * The ID field must be short and unique.
#   * Exactly one attr_path_regex must be a wildcard ("^[^\\.]*$").
#   * All non-wildcard attr_path_regexes must be mutually exclusive.
#   * Exactly one unversioned_attr_prefix must be the empty string.
#   * All non-wildcard unversioned_attr_prefixes must be mutually exclusive.
{
  by_name_dirs = [
    # Not quite yet!
    # {
    #   id = "py";
    #   path = "pkgs/development/python-modules/by-name";
    #   attr_path_regex = "^(python3\\d*Packages|python3\\d*.pkgs)\\..*$";
    #   unversioned_attr_prefix = "python3Packages";
    #   all_packages_path = "/pkgs/top-level/python-packages.nix";
    #   aliases_path = "/pkgs/top-level/python-aliases.nix";
    # }
    {
      id = "tcl";
      path = "pkgs/development/tcl-modules/by-name";
      attr_path_regex = "^(tcl\\d*Packages)\\..*$";
      unversioned_attr_prefix = "tclPackages";
      all_packages_path = "/pkgs/top-level/tcl-packages.nix";
    }
    {
      id = "main";
      path = "pkgs/by-name";
      attr_path_regex = "^[^\\.]*$";
      unversioned_attr_prefix = "";
      all_packages_path = "/pkgs/top-level/all-packages.nix";
      aliases_path = "/pkgs/top-level/aliases.nix";
    }
  ];
}
