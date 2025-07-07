# ssec-cli

## Introduction
SSEC was developed as a simple alternative to ZIP encryption for single files (analogous to gzip, but for encryption).
This repository contains a command line interface for encrypting and decrypting SSEC files.
To read more about the security guarantees of the SSEC file format go [here](https://github.com/james-conn/ssec-core).

## Installation
There are two recommended ways of installing `ssec-cli`.

### Installation Using Cargo
```sh
cargo install ssec-cli
```

### Installation Using Nix Flakes
```sh
nix profile install github:james-conn/ssec-cli
```

## Tutorial

### Encrypting A File (`enc`)
You can encrypt a file by using the `enc` subcommand:

```sh
ssec enc <IN_FILE> [OUT_FILE]
```

You will be prompted to input a password for the file interactively.
If `OUT_FILE` is not provided, the default value is the value of `IN_FILE` with a `.ssec` extension added to the end.

### Decrypting A File (`dec`)
You can decrypt a SSEC file by using the `dec` subcommand:

```sh
ssec dec <IN_FILE> <OUT_FILE>
```

You will be prompted to input the password for the file interactively.

### Decrypting A Remote File (`fetch`)
If you have a URL to a SSEC file, you can directly download the decrypted inner contents without needing to save the SSEC file itself:

```sh
ssec fetch <URL> <OUT_FILE>
```

You will be prompted to input the password for the file interactively.

### Silent Mode
All of the above subcommands will display a progress bar, if you would like to hide the progress bar you can pass in the `--silent` flag.
