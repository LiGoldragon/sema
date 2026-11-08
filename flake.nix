{
  description = "sema — Sema version 0: Markdown read structurally";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    ethos-zero = {
      url = "github:LiGoldragon/ethos-zero/04d93e4b705a1b2abbb806fa385a04ce74b13cdd";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, ethos-zero, rust-build }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;
        src = rust.cleanSource { root = ./.; };
        commonArguments = { inherit src; strictDeps = true; };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in
      {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          ethos = pkgs.runCommand "sema-ethos" {
            nativeBuildInputs = [ ethos-zero.packages.${system}.default ];
          } ''
            for file in ${./ethos}/*.ethos; do
              ethos-zero "Check.$file" | tee -a $out
            done
            ! grep -q '^Rejected' $out
          '';
          no-production-free-functions = pkgs.runCommand "sema-no-production-free-functions" { } ''
            if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' ${src}/src; then
              echo "production Rust must not use module-level free functions" >&2
              exit 1
            fi
            touch $out
          '';
          no-production-inherent-methods = pkgs.runCommand "sema-no-production-inherent-methods" { } ''
            if grep -R -n -E '^[[:space:]]*impl[[:space:]]+[[:alpha:]_][[:alnum:]_:<>]*[[:space:]]*\{' ${src}/src; then
              echo "production Rust must home behavior in traits" >&2
              exit 1
            fi
            touch $out
          '';
        };
        devShells.default = pkgs.mkShell {
          name = "sema";
          packages = [ toolchain ];
        };
      });
}
