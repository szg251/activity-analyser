{ pkgs, ... }:
{
  packages = [
    pkgs.cargo-watch
    pkgs.inferno
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  pre-commit.hooks = {
    rustfmt.enable = true;
    nixfmt-rfc-style = {
      enable = true;
      excludes = [ ".devenv.flake.nix" ];
    };
    clippy = {
      enable = true;
      settings.offline = false;
    };
  };

  enterTest = ''cargo test'';
}
