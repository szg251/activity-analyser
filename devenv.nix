{ pkgs, ... }:
{
  packages =
    [
      pkgs.cargo-watch
      pkgs.cargo-edit
      pkgs.clippy
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
      pkgs.mktemp
      pkgs.darwin.apple_sdk.frameworks.CoreFoundation
      pkgs.darwin.apple_sdk.frameworks.CoreServices
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  languages.rust.enable = true;

  pre-commit.hooks = {
    rustfmt = {
      enable = true;
      settings.manifest-path = "./backend/Cargo.toml";
    };
    nixfmt-rfc-style = {
      enable = true;
      excludes = [ ".devenv.flake.nix" ];
    };
    # clippy.enable = true;
  };
}
