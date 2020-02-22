## Pull Requests

All Pull Requests should first be made into the 'develop' branch, since the Github Actions CI badge build status that is shown in the README depends on the outcome of building Pull Requests from the 'develop' branch.

## Continuous Integration

Github Actions are used for Continuous Integration.
View the latest [CI Build Status](https://github.com/DataHighway-DHX/node/actions?query=workflow%3ACI+branch%3Adevelop) of the 'develop' branch, from which all Pull Requests are made into the 'master' branch.

Reference: https://help.github.com/en/actions/configuring-and-managing-workflows/configuring-a-workflow

## FAQ

* Question: Why do we need to install Rust Stable and Rust Nightly?
	* Answer: In .github/workflows/rust.yml, we need to run the following,
	because Substrate builds two binaries: 1) Wasm binary of your Runtime;
	and 2) Native executable containing all your other Substrate components
	including your runtimes too. The Wasm build requires rust nightly and
	wasm32-unknown-unknown to be installed. Note that we do not use
	`rustup update nightly` since the latest Rust Nightly may break our build,
	so we must manually change this to the latest Rust Nightly version only
	when it is known to work.
		```bash
		rustup toolchain install nightly-2020-02-17
		rustup update stable
		rustup target add wasm32-unknown-unknown --toolchain nightly
		```

* Question: Why do we install a specific version of Rust Nightly in the CI?
	* Answer: Since the latest version of Rust Nightly may break our build,
	and because developers may forget to update to the latest version of Rust
	Nightly locally. So the solution is to install a specific version of
	Rust Nightly in .github/workflows/rust.yml (i.e.
	`rustup toolchain install nightly-2020-02-17`), since for example
	the latest Rust Nightly version nightly-2020-02-20 may cause our CI tests
	to fail (i.e. https://github.com/DataHighway-DHX/node/issues/32)
