{
	description = "Alternating tiling layout for Sway, in Rust.";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
		hooks = {
			url = "github:cachix/git-hooks.nix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = {
		self,
		hooks,
		fenix,
		nixpkgs,
		...
	}: let
		systems = ["aarch64-linux" "x86_64-linux"];
		forAllSystems = f:
			nixpkgs.lib.genAttrs systems (system:
					f {
						pkgs =
							import nixpkgs {
								inherit system;
								overlays = [self.overlays.default];
							};
						inherit system;
						check = self.checks.${system}.pre-commit-check;
					});
	in {
		overlays.default = final: prev: {
			rustToolchain = with fenix.packages.${prev.stdenv.hostPlatform.system};
				combine (with complete; [clippy rustc cargo rustfmt rust-src]);
		};

		checks =
			forAllSystems ({
					pkgs,
					system,
					...
				}: {
					pre-commit-check =
						hooks.lib.${system}.run {
							src = ./.;
							package = pkgs.prek;
							hooks = {
								convco.enable = true;
								alejandra.enable = true;
								statix = {
									enable = true;
									settings.ignore = ["/.direnv"];
								};
								clippy.enable = true;
								rustfmt.enable = true;
							};
						};
				});

		packages =
			forAllSystems ({pkgs, ...}: {
					default =
						(pkgs.makeRustPlatform {
								cargo = pkgs.rustToolchain;
								rustc = pkgs.rustToolchain;
							}).buildRustPackage {
							pname = "swayalt";
							version = "0.1.0";
							src = ./.;
							meta.mainProgram = "swayalt";
							cargoLock.lockFile = ./Cargo.lock;
						};
				});

		devShells =
			forAllSystems ({
					pkgs,
					system,
					check,
					...
				}: {
					default =
						pkgs.mkShell {
							inherit (check) shellHook;
							buildInputs = check.enabledPackages;

							packages = with pkgs; [
								rustToolchain
								pkg-config
								cargo-deny
								cargo-edit
								cargo-semver-checks
								cargo-watch
								rust-analyzer
								bacon
							];

							env.RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
						};
				});
	};
}
