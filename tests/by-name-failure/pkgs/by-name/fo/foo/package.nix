{ }:
# If we caused an actual Nix failure
builtins.trace "This should be on stderr!"
throw "This is an error!"
