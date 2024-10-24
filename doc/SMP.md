# About how to boot Starry and ArceOS currently

* only work in x86_64
* refer to [README.md](../README.md) for the preparation of `disk.img`.

## Guest VM images and configs

* ArceOS binary image
    * repo: https://github.com/arceos-hypervisor/arceos/tree/gvm_sleep
    * build: 
        * `make A=examples/helloworld build`
    * copy image: 
        * `sudo cp /PATH/TO/arceos/examples/helloworld/helloworld_x86_64-qemu-q35.bin DISK/MOUNT/ON/tmp/arceos-x86-sleep.bin`
    * config file for vmm available at [arceos-x86_64-sleep.toml](../arceos-vmm/configs/arceos-x86_64-sleep.toml)
    * Note: ArceOS use COM1 at **0x2f8** for serial output.

* Starry binary image
    * repo: https://github.com/arceos-org/starry-next/tree/tick_loop
    * build: 
        * `./scripts/get_deps.sh`
        * `make user_apps`
        * `make ARCH=x86_64 build`
    * copy image: 
        * `sudo cp /PATH/TO/starry-next/starry-next_x86_64-qemu-q35.bin DISK/MOUNT/ON/tmp/starry-x86_64.bin`
    * config file for vmm available at [starry-x86_64.toml](../arceos-vmm/configs/starry-x86_64.toml)

## How to run

* open first terminal
    * `cd arceos-vmm`
    * `make ARCH=x86_64 ACCEL=y VM_CONFIGS=configs/arceos-x86_64-sleep.toml:configs/starry-x86_64.toml SMP=2 SECOND_SERIAL=y run`
    * ArceOS-hypervisor itself and Starry-next will print to this terminal.
    * `SECOND_SERIAL=y` will make qemu open a second serial port and listen on the socket interface from localhost constrained by the `TELNET_PORT` variable (default is 4321, currently only valid under `qemu_system_x86_64`)
        ```bash
        qemu-system-x86_64: -serial telnet:localhost:4321,server: info: QEMU waiting for connection on: disconnected:telnet:127.0.0.1:4321,server=on
        ```

* open another terminal
    * `telnet localhost 4321`
    * ArceOS as guest VM will print to this terminal.