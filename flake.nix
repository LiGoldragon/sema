{
  description = "sema — Sema version 0: Markdown read structurally";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    ethos-zero = {
      url = "github:LiGoldragon/ethos-zero/v22.0.0";
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
        ethosZero = ethos-zero.packages.${system}.default;
        # The hand-written source: trait bodies only; nothing generated is kept.
        handSource = rust.cleanSource { root = ./.; };
        # The Rust ethos-zero generates from the ethos, one module per file.
        generated = pkgs.runCommand "sema-generated" { nativeBuildInputs = [ ethosZero ]; } ''
          mkdir $out
          for file in ${./ethos}/*.ethos; do
            ethos-zero "Generate.{ $file $out }"
          done
        '';
        # The crate as it is built: the hand-written source with the generated modules beside it.
        src = pkgs.runCommand "sema-source" { } ''
          cp -r ${handSource} $out
          chmod -R u+w $out
          cp ${generated}/*.rs $out/src/
        '';
        commonArguments = { inherit src; strictDeps = true; };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in
      {
        packages.generated = generated;
        # The CLI speaks text, so it is built with the datom kinds.
        packages.default = craneLib.buildPackage (commonArguments // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--locked --features datom";
        });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets --features datom -- -D warnings";
          });
          # No type or trait is declared by hand: every one is generated from the ethos.
          no-hand-written-declarations = pkgs.runCommand "sema-no-hand-written-declarations" { } ''
            if grep -R -n -E '^[[:space:]]*(pub(\([^)]*\))? )?(struct|enum|trait|type|union) ' ${handSource}/src ${handSource}/cli; then
              echo "every type and trait is declared in the ethos" >&2
              exit 1
            fi
            touch $out
          '';
          datom = craneLib.cargoBuild (commonArguments // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--locked --features datom";
          });
          ethos = pkgs.runCommand "sema-ethos" {
            nativeBuildInputs = [ ethosZero ];
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
