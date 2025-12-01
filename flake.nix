{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-25.11";
		flake-utils.url = "github:numtide/flake-utils";
		crane.url = "github:ipetkov/crane";
	};

	outputs = { self, nixpkgs, flake-utils, crane }:
		flake-utils.lib.eachDefaultSystem (system:
			let pkgs = import nixpkgs {
				inherit system;
			};
			craneLib = (crane.mkLib pkgs);
			ssec = craneLib.buildPackage {
				pname = "ssec";
				src = craneLib.cleanCargoSource ./.;

				nativeBuildInputs = [ pkgs.installShellFiles ];

				SHELLS_OUTPUT = "share/completions";

				preBuildPhases = [ "mkCompletionsDir" ];
				mkCompletionsDir = "mkdir -p $out/share/completions";

				preInstallPhases = [ "installShellCompletions" ];
				# this runs both where we would expect but also in crane's inner derivation where our
				# `build.rs` is not run, the if test is a scuffed way of ensuring this won't run in
				# crane's inner derivation where `build.rs` isn't run
				installShellCompletions = ''
					if [ -f $out/share/completions/ssec.bash ]; then
						installShellCompletion\
							--bash $out/share/completions/ssec.bash\
							--zsh $out/share/completions/_ssec\
							--fish $out/share/completions/ssec.fish
					fi
				'';
			}; in {
				devShell = pkgs.mkShell {
					packages = with pkgs; [
						cargo
						clippy
					];
				};

				packages.default = ssec;
		});
}
