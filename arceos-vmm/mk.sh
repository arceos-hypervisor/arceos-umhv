rm disk.img
make disk_img
mkdir -p tmp
sudo mount disk.img tmp
# Copy guest OS binary image file.
sudo cp ../../guest/nimbos/kernel/target/aarch64/release/nimbos.bin tmp/nimbos-aarch64.bin
# sudo cp ../guest/testos/build/kernel.bin tmp/testos-aarch64.bin
# sudo cp ../guest/dtb/nimbos-aarch64.dtb tmp/

sudo umount tmp