IMAGE_FILE = ../linux-aarch64.bin
GITHUB_URL = https://github.com/luodeb/rk3588-linux-bin/raw/main/linux-aarch64.bin

download:
	@if [ ! -e $(IMAGE_FILE) ]; then \
		echo "linux-aarch64.bin not found, downloading from $(GITHUB_URL)..."; \
		curl -L -o $(IMAGE_FILE) $(GITHUB_URL); \
	else \
		echo "linux-aarch64.bin already exists."; \
	fi

rk3588.dtb:
	dtc -I dts -O dtb -o ../rk3588.dtb ../rk3588.dts

kernel: download rk3588.dtb build 
	./tools/rk3588/mkimg --dtb rk3588-firefly-itx-3588j.dtb --img $(OUT_BIN)
	@echo 'Built the FIT-uImage boot.img'
