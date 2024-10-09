
    .section .data
    .global guestkernel_start
    .global guestkernel_end
    .align 16
guestkernel_start:
    .incbin "../linux-aarch64.bin"
guestkernel_end:

  .section .data
    .global guestdtb_start
    .global guestdtb_end
    .align 16
guestdtb_start:
    .incbin "../rk3588.dtb"
guestdtb_end:
