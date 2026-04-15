{
  description = "sema — universal typed binary format";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";

    # The compiler pipeline
    askicc = {
      url = "github:LiGoldragon/askicc";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
    };
    askic = {
      url = "github:LiGoldragon/askic";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.askicc.follows = "askicc";
    };
    semac = {
      url = "github:LiGoldragon/semac";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.askic.follows = "askic";
    };
  };

  outputs = { self, nixpkgs, fenix, crane, askicc, askic, semac, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

    in {
      packages.${system} = {
        askicc = askicc.packages.${system}.askicc;
        synth-dialect = askicc.packages.${system}.synth-dialect;
        askic = askic.packages.${system}.askic;
        semac = semac.packages.${system}.semac;
      };

      checks.${system} = {
        # Bootstrap
        askicc-build = askicc.checks.${system}.build;
        askicc-tests = askicc.checks.${system}.cargo-tests;

        # Compiler
        askic-build = askic.checks.${system}.build;
        askic-tests = askic.checks.${system}.cargo-tests;

        # Sema generator
        semac-build = semac.checks.${system}.build;
        semac-tests = semac.checks.${system}.cargo-tests;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          askicc.packages.${system}.askicc
          askic.packages.${system}.askic
          semac.packages.${system}.semac
          pkgs.jujutsu
        ];
        SYNTH_DIR = "${askicc.packages.${system}.synth-dialect}";
      };
    };
}
