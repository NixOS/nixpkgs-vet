let
  valid = [
    (lib.lists.optional condition value)
    (lib.lists.optionals condition [ value ])
    (lib.optional condition value)
    (lib.optionals condition [ value ])
    (optional condition value)
    (optionals condition [ value ])
  ];

  invalid = [
    (lib.lists.optional condition [ value ])
    (lib.lists.optional condition [
      value # Preserve this comment
    ])
    (lib.lists.optional (lib.lists.optional condition value) [ value ])
    (lib.optional condition [ value ])
    (lib.optional condition [
      value # Preserve this comment
    ])
    (lib.optional (lib.optional condition value) [ value ])
    (optional condition [ value ])
    (optional condition [
      value # Preserve this comment
    ])
    (optional (optional condition value) [ value ])
  ];
in
import <test-nixpkgs> { root = ./.; }
