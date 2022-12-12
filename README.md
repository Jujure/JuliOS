# JuliOS

[![Build Status](https://drone.juju.re/api/badges/juju/JuliOS/status.svg)](https://drone.juju.re/juju/JuliOS)

Just an Unstable, Lame and Ineffective Operating System

# Build requirements

* rust nightly
* rust-src (`rustup component add rust-src`)
* grub2
* xorriso

# Build

```sh
make        # The kernel and an ISO
make run    # Run qemu on the ISO
make debug  # Run bochs on the ISO
make clean  # Clean everything
```
