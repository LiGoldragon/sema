{
  description = "sema-codec-measure: Markdown libraries measured as sema's codec";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  outputs = { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {
      packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {
        pname = "sema-codec-measure";
        version = "0.0.0";
        src = ./..;
        cargoRoot = "measure";
        buildAndTestSubdir = "measure";
        cargoLock.lockFile = ./Cargo.lock;
        doCheck = false;
      };
    };
}
