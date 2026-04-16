{
  description = "sema — universal typed binary format";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";

    # The pipeline — each stage depends on the previous
    corec = {
      url = "github:LiGoldragon/corec";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
    };
    aski-core = {
      url = "github:LiGoldragon/aski-core";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.corec.follows = "corec";
    };
    sema-core = {
      url = "github:LiGoldragon/sema-core";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.corec.follows = "corec";
    };
    askicc = {
      url = "github:LiGoldragon/askicc";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.aski-core.follows = "aski-core";
    };
    askic = {
      url = "github:LiGoldragon/askic";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.aski-core.follows = "aski-core";
      inputs.sema-core.follows = "sema-core";
      inputs.askicc.follows = "askicc";
    };
    semac = {
      url = "github:LiGoldragon/semac";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.sema-core.follows = "sema-core";
    };
  };

  outputs = { self, nixpkgs, fenix, crane, flake-utils,
              corec, aski-core, sema-core, askicc, askic, semac, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages = {
          corec = corec.packages.${system}.corec;
          aski-core = aski-core.packages.${system}.source;
          sema-core = sema-core.packages.${system}.source;
          askicc = askicc.packages.${system}.askicc;
          dialect-data = askicc.packages.${system}.dialect-data;
          askic = askic.packages.${system}.askic;
          semac = semac.packages.${system}.semac;
        };

        checks = {
          # Stage 1: corec
          corec-tests = corec.checks.${system}.tests;

          # Stage 2a: aski-core
          aski-core-lib = aski-core.checks.${system}.lib-build;

          # Stage 2b: sema-core
          sema-core-lib = sema-core.checks.${system}.lib-build;

          # Stage 3: askicc
          askicc-build = askicc.checks.${system}.build;
          askicc-tests = askicc.checks.${system}.tests;

          # Stage 4: askic
          askic-build = askic.checks.${system}.build;
          askic-tests = askic.checks.${system}.tests;

          # Stage 5: semac
          semac-build = semac.checks.${system}.build;
          semac-tests = semac.checks.${system}.tests;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            corec.packages.${system}.corec
            askicc.packages.${system}.askicc
            askic.packages.${system}.askic
            semac.packages.${system}.semac
            pkgs.jujutsu
          ];
        };
      }
    );
}
