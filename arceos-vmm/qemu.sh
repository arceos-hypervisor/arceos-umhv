qemu-system-aarch64 -machine virt,virtualization=on,gic-version=2 -m 1G -cpu cortex-a57 -smp 1 \
        -append "earlycon console=ttyAMA0 root=/dev/vda rw audit=0 default_hugepagesz=32M hugepagesz=32M hugepages=4"\
        -display none -global virtio-mmio.force-legacy=false -kernel configs/linux-6.6.62.bin \
        -netdev user,id=n0,hostfwd=tcp::5555-:22 -device virtio-net-device,bus=virtio-mmio-bus.24,netdev=n0 \
        -drive file=/home/hky/workspace/Linux/ubuntu-22.04-rootfs_ext4.img,if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.25 \
        -serial mon:stdio
