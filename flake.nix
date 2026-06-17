{
  description = "zellij-attention — 3-state Claude tab indicator (Zellij WASM plugin)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      # Rust toolchain (default profile → rustc/cargo/clippy/rustfmt) plus the
      # wasm target Zellij plugins compile to.
      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = ["wasm32-wasip1"];
        extensions = ["rust-analyzer" "rust-src"];
      };

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

      # Host Rust triple (e.g. aarch64-apple-darwin) for running native unit tests.
      # .cargo/config.toml pins builds to wasm32-wasip1, so the test/clippy commands
      # must pass --target explicitly or they'd produce a non-runnable wasm binary.
      hostTarget = pkgs.stdenv.hostPlatform.rust.rustcTarget;

      # zellij-tile's *host* (non-wasm) dependency graph pulls in openssl-sys and
      # curl-sys via zellij-utils. These are only needed to compile/run the unit
      # tests + clippy (host target) — never for the wasm artifact itself.
      hostNativeInputs = [pkgs.pkg-config];
      hostBuildInputs = [pkgs.openssl pkgs.curl];
    in {
      devShells.default = pkgs.mkShell {
        # rust → build/test; pkg-config + openssl + curl → host-target tests/clippy.
        packages = [rust] ++ hostNativeInputs ++ hostBuildInputs;
        shellHook = ''
          echo "zellij-attention dev shell"
          echo "  cargo build --release              → target/wasm32-wasip1/release/zellij-attention.wasm"
          echo "  cargo test --target ${hostTarget}  → unit tests (host target)"
        '';
      };

      # Builds the wasm artifact for nixfiles consumption.
      #
      # buildRustPackage's build hook passes an explicit `--target <host>`, which
      # would build the heavy openssl/curl host graph (gated out of zellij-utils
      # for wasm) and overrides both CARGO_BUILD_TARGET and .cargo/config.toml. So
      # we drive the build ourselves with --target wasm32-wasip1; cargoSetupHook
      # still vendors the crates, so --offline resolves them with no network.
      # A custom installPhase copies the wasm (the default install can't find a
      # non-host artifact). doCheck is off — wasm unit tests can't run natively.
      packages.default = rustPlatform.buildRustPackage {
        pname = "zellij-attention";
        version = "0.4.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        doCheck = false;
        dontStrip = true; # wasm artifact — skip host strip

        buildPhase = ''
          runHook preBuild
          cargo build --release --offline --target wasm32-wasip1
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p $out/bin
          cp target/wasm32-wasip1/release/zellij-attention.wasm $out/bin/
          runHook postInstall
        '';
      };

      # Debug wasm — same as packages.default but a debug profile, so the
      # #[cfg(debug_assertions)] eprintln tracing (pipe arrivals, rename decisions,
      # focus clears) is compiled in and shows up in the zellij log. For live diagnosis.
      packages.debug = rustPlatform.buildRustPackage {
        pname = "zellij-attention-debug";
        version = "0.4.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        doCheck = false;
        dontStrip = true;

        buildPhase = ''
          runHook preBuild
          cargo build --offline --target wasm32-wasip1
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p $out/bin
          cp target/wasm32-wasip1/debug/zellij-attention.wasm $out/bin/
          runHook postInstall
        '';
      };

      # Host-target build that runs clippy (deny warnings) + the unit tests.
      # Separate from packages.default because tests need the native target and
      # the openssl/curl system libs. Run with `nix build .#checks.<system>.default`.
      #
      # The unit tests live in the lib crate (lib.rs: #[cfg(test)] mod tests), whose
      # single host-ABI import is stubbed in tests.rs. The bin (register_plugin!)
      # can't *link* on a host target — only wasm — so we never build it here:
      # clippy runs in check mode (no linking) and tests target --lib only.
      checks.default = rustPlatform.buildRustPackage {
        pname = "zellij-attention-checks";
        version = "0.4.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = hostNativeInputs;
        buildInputs = hostBuildInputs;

        buildPhase = ''
          runHook preBuild
          cargo clippy --release --all-targets --target ${hostTarget} -- -D warnings
          runHook postBuild
        '';

        doCheck = true;
        checkPhase = ''
          runHook preCheck
          cargo test --release --lib --target ${hostTarget}
          runHook postCheck
        '';

        installPhase = "touch $out";
      };
    });
}
