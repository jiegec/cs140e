cs140e
=======================================

My code for Stanford cs140e. Use `./gendiff.sh` to generate diff from subdirectories and commit into this git repo.

Using `alias code-rust="env RUST_TARGET_PATH=(pwd) RUST_LOG=rls=debug CARGO_INCREMENTAL=0 code-insiders"` in my fish config.

```
rustup toolchain install nightly-2018-03-1
rustup component add rustfmt-preview rls-preview rust-src rust-analysis
```

Build bootloader first, then

```
cp os/bootloader/build/bootloader.bin /Volumes/boot/kernel8.img
cp os/bootloader/ext/config.txt /Volumes/boot/config.txt
cd os/kernel
make install; and screen /dev/tty.SLAB_USBtoUART 115200
```

To run kernel in QEMU, build kernel and then run it:

```
cd os/kernel
make QEMU=1
qemu-system-aarch64 -machine raspi3 -serial null -serial mon:stdio -kernel build/kernel.bin
```

