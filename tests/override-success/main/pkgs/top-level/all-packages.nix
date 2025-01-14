self: super: { foo = self.callPackage ./../by-name/fo/foo/package.nix { enableBar = true; }; }
