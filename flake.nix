{
  description = "sema — workspace typed-database kernel (redb + rkyv + version-skew guard)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        checks = {
          # ─── Build ───────────────────────────────────────────
          # The library compiles (separate from running tests).
          build = craneLib.cargoBuild (commonArgs // {
            inherit cargoArtifacts;
          });

          # ─── Default test surface ─────────────────────────────
          # `cargo test` — runs every test target end to end.
          # Includes the legacy slot-store tests and the new
          # kernel-surface tests in one pass.
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });

          # ─── Per-file integration test runs ──────────────────
          # Each integration test file gets its own check so a
          # failure surfaces named, not buried.
          test-legacy-slot-store = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test sema";
          });

          test-kernel-surface = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test kernel";
          });

          # ─── Doc-tests ────────────────────────────────────────
          # The kernel's typed-Table example doctests in lib.rs
          # are `ignore`d (they reference types not in scope at
          # doc-test time) but the prose still has to compile to
          # valid markdown / valid rustdoc syntax. `cargo test
          # --doc` enforces that.
          test-doc = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--doc";
          });

          # ─── Documentation builds without warnings ────────────
          # rustdoc catches broken intra-doc links, missing
          # references, malformed `[`...`]` brackets in prose.
          # Sema's API surface IS partly documentation (the
          # `Schema` and `Table<K, V>` examples drive consumer
          # crates).
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });

          # ─── Formatter ────────────────────────────────────────
          fmt = craneLib.cargoFmt {
            inherit src;
          };

          # ─── Lint ─────────────────────────────────────────────
          # Clippy on the whole crate; warnings are errors.
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };

        devShells.default = pkgs.mkShell {
          name = "sema";
          packages = [
            pkgs.jujutsu
            pkgs.pkg-config
            toolchain
          ];
        };
      }
    );
}
