{
  description = "Haruna's stories web app";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    devenv.url = "github:cachix/devenv";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs = inputs@{ nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {

      imports = [
        inputs.devenv.flakeModule
        inputs.nci.flakeModule
      ];

      systems = [ "x86_64-linux" "x86_64-darwin" ];

      perSystem = { config, self', pkgs, system, ... }:
        let
          isDarwin = pkgs.lib.hasSuffix "darwin" system;
          buildInputs =
            if isDarwin
            then [
              pkgs.mktemp
              pkgs.darwin.apple_sdk.frameworks.CoreFoundation
              pkgs.darwin.apple_sdk.frameworks.CoreServices
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ] else [ ];
          nciPkgs = config.nci.outputs."activity-analyser".packages;
        in
        {
          devenv.shells.default = {
            packages = [ pkgs.cargo-watch ] ++ buildInputs;
            languages.rust.enable = true;
            pre-commit.hooks = {
              rustfmt.enable = true;
              nixpkgs-fmt.enable = true;
            };
          };
          nci = {
            projects."activity-analyser".path = ./.;
            crates."activity-analyser" = {
              depsDrvConfig = { inherit buildInputs; };
              drvConfig = {
                inherit buildInputs;
                env = {
                  SQLX_OFFLINE = "true";
                };
              };
            };
          };
          packages.default = nciPkgs.release;
        };
    };
}
