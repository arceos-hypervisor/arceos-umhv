fada: build
	gzip -9 -cvf $(OUT_BIN) > arceos-fada.bin.gz
	mkimage -f configs/bsta1000b-fada-arceos.its arceos-fada.itb
	@echo 'Built the FIT-uImage arceos-fada.itb'
