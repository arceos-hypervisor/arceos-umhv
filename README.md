<!-- <div align="center">

<img src="https://arceos-hypervisor.github.io/doc/assets/logo.svg" alt="axvisor-logo" width="64">

</div> -->

<h2 align="center">AxVisor</h1>

<p align="center">A unified modular hypervisor based on ArceOS.</p>

<div align="center">

[![GitHub stars](https://img.shields.io/github/stars/arceos-hypervisor/axvisor?logo=github)](https://github.com/arceos-hypervisor/axvisor/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/arceos-hypervisor/axvisor?logo=github)](https://github.com/arceos-hypervisor/axvisor/network)
[![license](https://img.shields.io/github/license/arceos-hypervisor/axvisor)](https://github.com/arceos-hypervisor/axvisor/blob/master/LICENSE)

</div>

English | [中文版](README_CN.md)

# Introduction

AxVisor is a hypervisor implemented based on the ArceOS unikernel framework. Its goal is to leverage the foundational operating system features provided by ArceOS to implement a unified modular hypervisor.

"Unified" refers to using the same codebase to support x86_64, Arm (aarch64), and RISC-V architectures simultaneously, in order to maximize the reuse of architecture-independent code and simplify development and maintenance costs.

"Modular" means that the functionality of the hypervisor is decomposed into multiple modules, each implementing a specific function. The modules communicate with each other through standard interfaces to achieve decoupling and reuse of functionality.

## Architecture

The software architecture of AxVisor is divided into five layers as shown in the diagram below. Each box represents an independent module, and the modules communicate with each other through standard interfaces.

![Architecture](https://arceos-hypervisor.github.io/doc/assets/arceos-hypervisor-architecture.png)

The complete architecture description can be found in the [documentation](https://arceos-hypervisor.github.io/doc/arch_cn.html).

## Hardwares

Currently, AxVisor has been verified on the following platforms:

- [x] QEMU ARM64 virt (qemu-max)
- [x] Rockchip RK3568 / RK3588
- [x] 黑芝麻华山 A1000

## Guest VMs

Currently, AxVisor has been verified in scenarios with the following systems as guests:

* [ArceOS](https://github.com/arceos-org/arceos)
* [Starry-OS](https://github.com/Starry-OS)
* [NimbOS](https://github.com/equation314/nimbos)
* Linux
  * currently only Linux with passthrough device on aarch64 is tested.
  * single core: [config.toml](configs/vms/linux-qemu-aarch64.toml) | [dts](configs/vms/linux-qemu.dts)
  * smp: [config.toml](configs/vms/linux-qemu-aarch64-smp2.toml) | [dts](configs/vms/linux-qemu-smp2.dts)

# Build and Run

Use the command `git clone https://github.com/arceos-hypervisor/axvisor.git` to pull the AxVisor source code.

## Build Environment

AxVisor is written in the Rust programming language, so you need to install the Rust development environment following the instructions on the official Rust website. Additionally, you need to install cargo-binutils to use tools like rust-objcopy and rust-objdump.
```console
$ cargo install cargo-binutils
```

If necessary, you may also need to install [musl-gcc](http://musl.cc/x86_64-linux-musl-cross.tgz) to build guest applications.

## Prepare the Guest VM

After AxVisor starts, it loads and starts the guest based on the information in the guest configuration file. Currently, AxVisor supports loading guest images from a FAT32 file system and also supports binding guest images to the hypervisor image through static compilation (using include_bytes).

### Configuration Files

Since configuring the guest is a complex process, AxVisor chooses to use TOML files to manage the guest configurations. These configurations include the virtual machine ID, virtual machine name, virtual machine type, number of CPU cores, memory size, virtual devices, passthrough devices, and more. In the source code, the `./config/vms` directory contains some example templates for guest configurations.

In addition, you can use the [axvmconfig](https://github.com/arceos-hypervisor/axvmconfig) tool to generate a custom configuration file. For detailed information, refer to the [axvmconfig](https://arceos-hypervisor.github.io/axvmconfig/axvmconfig/index.html) documentation.

### Load from file system

1. Building your own guest machine image file

2. Modify the configuration items in the corresponding `./configs/vms/<ARCH_CONFIG>.toml`
     - `image_location="fs"` indicates loading from the file system.
     - `kernel_path` specifies the path to the kernel image in the file system.
     - others

3. Create a disk image file and place the guest machine image into the file system.

  1. Use the `make disk_img` command to generate an empty FAT32 disk image file named `disk.img`.
  2. Manually mount `disk.img`, and then place your guest machine image into the file system.

      ```console
      $ mkdir -p tmp
      $ sudo mount disk.img tmp
      $ sudo cp /PATH/TO/YOUR/GUEST/VM/IMAGE tmp/
      $ sudo umount tmp
      ```

4. When building AxVisor, the APP_FEATURES=fs option is required.

### Load from memory

1. Building your own guest machine image file

2. Modify the configuration items in the corresponding `./configs/vms/<ARCH_CONFIG>.toml`
     - `image_location="memory"` indicates loading from the memory.
     - `kernel_path` kernel_path specifies the relative/absolute path of the kernel image in the workspace.
     - others

3. Currently, the method of statically compiling and binding the guest machine image only supports loading one guest machine image.

## Build and Run

Depending on the chosen method for loading the guest machine image, the following commands may need to be modified accordingly!

### x86_64 for nimbos

1. `make ARCH=x86_64 defconfig`
2. `make ACCEL=y ARCH=x86_64 LOG=info VM_CONFIGS=configs/vms/nimbos-x86_64.toml APP_FEATURES=fs run`

### aarch64 for nimbos

1. `make ARCH=aarch64 defconfig`
2. `make ACCEL=n ARCH=aarch64 LOG=info VM_CONFIGS=configs/vms/nimbos-aarch64.toml APP_FEATURES=fs run`

### riscv64 for nimbos

1. `make ARCH=aarch64 defconfig`
2. `make ACCEL=n ARCH=riscv64 LOG=info VM_CONFIGS=configs/vms/nimbos-riscv64.toml APP_FEATURES=fs run`

### aarch64 for Linux

You need to modify the phys-memory-size in configs/platforms/aarch64-qemu-virt-hv.toml to 0x2_0000_0000 (8G).

1. `make ARCH=aarch64 defconfig`
2. `make ARCH=aarch64 VM_CONFIGS=configs/vms/linux-qemu-aarch64.toml LOG=debug BUS=mmio NET=y FEATURES=page-alloc-64g MEM=8g run`

### aarch64 for Linux SMP=2

1. `make ARCH=aarch64 defconfig`
2. `make ARCH=aarch64 VM_CONFIGS=configs/vms/linux-qemu-aarch64-smp2.toml LOG=debug BUS=mmio NET=y  BLK=y SMP=2 FEATURES=page-alloc-64g MEM=8g run`

## Demo

```console
Booting from ROM..
Initialize IDT & GDT...

       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = x86_64
platform = x86_64-qemu-q35
target = x86_64-unknown-none
smp = 1
build_mode = release
log_level = warn

Starting virtualization...
Running guest...

NN   NN  iii               bb        OOOOO    SSSSS
NNN  NN       mm mm mmmm   bb       OO   OO  SS
NN N NN  iii  mmm  mm  mm  bbbbbb   OO   OO   SSSSS
NN  NNN  iii  mmm  mm  mm  bb   bb  OO   OO       SS
NN   NN  iii  mmm  mm  mm  bbbbbb    OOOO0    SSSSS
              ___    ____    ___    ___
             |__ \  / __ \  |__ \  |__ \
             __/ / / / / /  __/ /  __/ /
            / __/ / /_/ /  / __/  / __/
           /____/ \____/  /____/ /____/

arch = x86_64
platform = rvm-guest-x86_64
build_mode = release
log_level = warn

Initializing kernel heap at: [0xffffff800028ed00, 0xffffff800068ed00)
Initializing IDT...
Loading GDT for CPU 0...
Initializing frame allocator at: [PA:0x68f000, PA:0x1000000)
Mapping .text: [0xffffff8000200000, 0xffffff800021b000)
Mapping .rodata: [0xffffff800021b000, 0xffffff8000220000)
Mapping .data: [0xffffff8000220000, 0xffffff800028a000)
Mapping .bss: [0xffffff800028e000, 0xffffff800068f000)
Mapping boot stack: [0xffffff800028a000, 0xffffff800028e000)
Mapping physical memory: [0xffffff800068f000, 0xffffff8001000000)
Mapping MMIO: [0xffffff80fec00000, 0xffffff80fec01000)
Mapping MMIO: [0xffffff80fed00000, 0xffffff80fed01000)
Mapping MMIO: [0xffffff80fee00000, 0xffffff80fee01000)
Initializing drivers...
Initializing Local APIC...
Initializing HPET...
HPET: 100.000000 MHz, 64-bit, 3 timers
Calibrated TSC frequency: 2993.778 MHz
Calibrated LAPIC frequency: 1000.522 MHz
Initializing task manager...
/**** APPS ****
cyclictest
exit
fantastic_text
forktest
forktest2
forktest_simple
forktest_simple_c
forktree
hello_c
hello_world
matrix
sleep
sleep_simple
stack_overflow
thread_simple
user_shell
usertests
yield
**************/
Running tasks...
test kernel task: pid = TaskId(2), arg = 0xdead
test kernel task: pid = TaskId(3), arg = 0xbeef
Rust user shell
>> hello_world
Hello world from user mode program!
Shell: Process 5 exited with code 0
>>
......
```

# Contributing

Feel free to fork this repository and submit a pull request.

You can refer to these [discussions]((https://github.com/arceos-hypervisor/axvisor/discussions)) to gain deeper insights into the project's ideas and future development direction.

## Development

AxVisor, as a modular hypervisor, has many components used as Crates. You can use the `tool/dev_env.py` command to localize the relevant Crates, making it easier for development and debugging.

## Contributors

This project exists thanks to all the people who contribute.

<a href="https://github.com/arceos-hypervisor/axvisor/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=arceos-hypervisor/axvisor" />
</a>

# License

AxVisor uses the following open-source license:

 * Apache License, Version 2.0
 * MulanPubL-2.0
 * MulanPSL2
 * GPL-3.0-or-later
