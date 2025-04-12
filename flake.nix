{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
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

				SHELLS_OUTPUT = "share/completions";

				preBuildPhases = [ "mkCompletions" ];
				mkCompletions = "mkdir -p $out/share/completions";

				preInstallPhases = [ "linkBashCompletions" ];
				# this runs both where we would expect but also in crane's inner derivation where our
				# `build.rs` is not run, the if test is a scuffed way of ensuring this won't run in
				# crane's inner derivation where `build.rs` isn't run
				linkBashCompletions = ''
					if [ -f $out/share/completions/ssec.bash ]; then
						mkdir -p $out/share/bash-completion/completions
						ln -s $out/share/completions/ssec.bash $out/share/bash-completion/completions/ssec
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
