{
  description = "virtual environments";

  inputs =
    {
      nixpkgs.url = github:nixos/nixpkgs/nixos-unstable-small;
      utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
      rust.url = "github:oxalica/rust-overlay";
    };



  outputs = inputs@{ self, nixpkgs, utils, rust }:
    let
      rust-analyzer-overlay = final: prev: {
        rust-analyzer = prev.rust-analyzer.override {
          rustSrc = "${final.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
        };
        rust-analyzer-unwrapped = final.rust-bin.nightly.latest.rust-analyzer-preview;
      };
    in
    utils.lib.mkFlake {
      inherit self inputs;

      sharedOverlays = [
        rust.overlay
        rust-analyzer-overlay
      ];

      outputsBuilder = channels:
        with import nix/dependencies.nix { pkgs = channels.nixpkgs; lib = nixpkgs.lib; };
        {
          # Evaluates to `devShell.<system> = <nixpkgs-channel-reference>.mkShell { name = "devShell"; };`.
          devShell = channels.nixpkgs.mkShell {
            name = "openvaf-devel";

            inherit buildInputs LLD_LIB_DIR LIBCLANG_PATH;

            nativeBuildInputs = with channels.nixpkgs; nativeBuildInputs ++ [
              /* rust-bin.stable.latest.default */
              rust-bin.nightly.latest.default
              rust-analyzer
              cargo-expand
              cargo-license
              cargo-deny
              crate2nix
              cargo-outdated
              cargo-edit
              cargo-flamegraph
              cargo-bloat
              cargo-deps
              graphviz
              tokei
              mdbook
              s3cmd
            ];

            NIX_CFLAGS_LINK = "-fuse-ld=lld";

          };
        };

    };
}