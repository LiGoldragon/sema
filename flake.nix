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
        scriptApplication = name: script: pkgs.writeShellApplication {
          name = "sema-${name}";
          runtimeInputs = [
            toolchain
          ];
          text = ''
            exec "${script}" "$@"
          '';
        };
        testScript = scriptApplication "test" ./scripts/test;
        testDocScript = scriptApplication "test-doc" ./scripts/test-doc;
        testKernelSurfaceScript = scriptApplication "test-kernel-surface" ./scripts/test-kernel-surface;
        testNoLegacySurfaceScript = scriptApplication "test-no-legacy-surface" ./scripts/test-no-legacy-surface;
      in
      {
        packages = {
          default = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });

          test = testScript;
          test-doc = testDocScript;
          test-kernel-surface = testKernelSurfaceScript;
          test-no-legacy-surface = testNoLegacySurfaceScript;
        };

        apps = {
          default = {
            type = "app";
            program = "${testScript}/bin/sema-test";
            meta.description = "Run sema's full test suite";
          };

          test = {
            type = "app";
            program = "${testScript}/bin/sema-test";
            meta.description = "Run sema's full test suite";
          };

          test-doc = {
            type = "app";
            program = "${testDocScript}/bin/sema-test-doc";
            meta.description = "Run sema's documentation tests";
          };

          test-kernel-surface = {
            type = "app";
            program = "${testKernelSurfaceScript}/bin/sema-test-kernel-surface";
            meta.description = "Run sema's typed-kernel integration tests";
          };

          test-no-legacy-surface = {
            type = "app";
            program = "${testNoLegacySurfaceScript}/bin/sema-test-no-legacy-surface";
            meta.description = "Run sema's retired-surface absence witnesses";
          };
        };

        checks = {
          # ─── Build ───────────────────────────────────────────
          # The library compiles (separate from running tests).
          build = craneLib.cargoBuild (commonArgs // {
            inherit cargoArtifacts;
          });

          # ─── Default test surface ─────────────────────────────
          # `cargo test` — runs every test target end to end.
          # Includes the kernel-surface tests and retired-surface
          # absence witnesses in one pass.
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });

          # ─── Per-file integration test runs ──────────────────
          # Each integration test file gets its own check so a
          # failure surfaces named, not buried.
          test-no-legacy-surface = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test no_legacy_surface";
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
