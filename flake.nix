{
  description = "Activity Analyser web app";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs = inputs@{ nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {

      imports = [
        inputs.nci.flakeModule
      ];

      systems = [ "x86_64-linux" "x86_64-darwin" ];

      perSystem = { config, self', pkgs, system, ... }:
        let
          crateName = "activity-analyser";

        in
        {
          nci = {
            toolchainConfig = {
              channel = "stable";
              components = [ "rust-analyzer" "rust-src" "clippy" "rustfmt" ];
            };
            projects.${crateName}.path = ./.;
            crates.${crateName} = {
              drvConfig = {
                env = {
                  SQLX_OFFLINE = "true";
                };
              };
            };
          };
          devShells.default = config.nci.outputs.${crateName}.devShell;
          packages.default = config.nci.outputs.${crateName}.packages.release;
          checks.default = config.nci.outputs.${crateName}.check;
        };
    };
}
