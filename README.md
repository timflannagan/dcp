# dcp: docker cp made easy

## Summary

Containers are great tools that can encapsulate an application and its dependencies,
allowing apps to run anywhere in a streamlined way. Some container images contain
commands to start a long-lived binary, whereas others may simply contain data
that needs to be available in the environment (for example, a Kubernetes cluster).
For example, the operator-framework `bundle` format is an example of using
containers image to store manifests, which can be unpacked on-cluster and made
available to end users.

One of the downsides of using container images to store data is that they are
necessarily opaque. There's no way to quickly tell what's inside the image, although
the hash digest is useful in seeing whether the image has changed from a previous
version. The options are to use `docker cp` or something similar using podman
or containerd.

Using `docker cp` by itself can be cumbersome. Say you have a remote image
somewhere in a registry. You have to pull the image, create a container from that
image, and only then run `docker cp <container-id>` using an unintuitive syntax for selecting
what should be copied to the local filesystem.

`dcp` is a simple binary that attempts to simplify this workflow. A user can simply
say `dcp <image-name>` and it can extract the contents of that image onto the
local filesystem. It can also just print the contents of the image to stdout, and
not create any local files.

## Installing 

### Download compiled binary

The [release section](https://github.com/exdx/dcp/releases) has a number
of precompiled versions of dcp for different platforms. Currently only Linux and
MacOS are pre-built. For MacOS, both arm and x86 targets are provided, and 
for Linux only x86 is provided. If your system is not supported, building dcp from
the source is straightforward.

### Build from source

To build from source, ensure that you have the rust toolchain installed locally.
This project does not rely on nightly and uses the 1.62-stable toolchain. 
Clone the repository and run `cargo build --release` to build a release version
of the binary. From there, you can move the binary to a folder on your $PATH to access
it easily. 

## Implementation

Because there wasn't a suitable `containerd` client implementation in Rust at the time
of writing, `dcp` relies on the docker APIs provided by an external crate. Unfortunately,
this limits `dcp` to only working on systems where docker is the container runtime.

## Flags and Examples

By default, dcp will copy content to the current directory `.`. For example, lets
try issuing the following command:

```
$ dcp tyslaton/sample-catalog:v0.0.4 -p configs
```

This command will copy the `configs` directory (specified via the `-p` flag) from the image to the current directory.

For further configuration, lets try:

```
$ dcp tyslaton/sample-catalog:v0.0.4 -d output -p configs
```

This command pulls down the requested image, only extracting
the `configs` directory and copying it to the `output` directory
locally (specified via the `-d` flag). 

## Testing
If you would like to run the test suite, you just need to run the standard cargo command. This will run all relevant unit, integration and documentation tests.

```
$ cargo test
```

> **Note**: In the current state of the repo running this command will write test output to `/tmp`. This output will get cleaned up automatically on shutdown. Automatically cleaning up this output was considered, however, leaving the output was favorable. In the future this project will use an in-memory file system to avoid this issue entirely.
