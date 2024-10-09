{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    openssl.dev
    pkg-config
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    channel = "stable";
    enable = true;
    mold.enable = true;
  };

  env.NIX_ENFORCE_PURITY = 0;
  env.RUST_BACKTRACE = "full";

  enterShell = ''
      echo "Rust version: $(rustc --version)"
      echo "Cargo version: $(cargo --version)"
      echo "RUST_SRC_PATH: $RUST_SRC_PATH"
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  pre-commit.hooks = {
      # some hooks have more than one package, like clippy:
      clippy.enable = true;
      # some hooks provide settings
      clippy.settings.allFeatures = true;
    };
}
