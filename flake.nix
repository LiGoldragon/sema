{
  description = "sema — universal typed binary format";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";

    # The three compiler stages
    synthc = {
      url = "github:LiGoldragon/synthc";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
    };
    askic = {
      url = "github:LiGoldragon/askic";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.synthc.follows = "synthc";
    };
    semac = {
      url = "github:LiGoldragon/semac";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.crane.follows = "crane";
      inputs.askic.follows = "askic";
    };
  };

  outputs = { self, nixpkgs, fenix, crane, synthc, askic, semac, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

    in {
      packages.${system} = {
        synthc = synthc.packages.${system}.synthc;
        synth-dialect = synthc.packages.${system}.synth-dialect;
        askic = askic.packages.${system}.askic;
        semac = semac.packages.${system}.semac;
      };

      checks.${system} = {
        # Stage 1
        synthc-build = synthc.checks.${system}.build;
        synthc-tests = synthc.checks.${system}.cargo-tests;

        # Stage 2
        askic-build = askic.checks.${system}.build;
        askic-tests = askic.checks.${system}.cargo-tests;

        # Stage 3
        semac-build = semac.checks.${system}.build;
        semac-tests = semac.checks.${system}.cargo-tests;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          synthc.packages.${system}.synthc
          askic.packages.${system}.askic
          semac.packages.${system}.semac
          pkgs.jujutsu
        ];
        SYNTH_DIR = "${synthc.packages.${system}.synth-dialect}";
      };
    };
}
