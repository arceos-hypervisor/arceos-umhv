pub mod traps {
    pub mod interrupt {
        pub const VIRTUAL_SUPERVISOR_SOFT: usize = 1 << 2;
        pub const VIRTUAL_SUPERVISOR_TIMER: usize = 1 << 6;
        pub const VIRTUAL_SUPERVISOR_EXTERNAL: usize = 1 << 10;
    }

    pub mod exception {
        pub const INST_ADDR_MISALIGN: usize = 1 << 0;
        pub const ILLEGAL_INST: usize = 1 << 2;
        pub const BREAKPOINT: usize = 1 << 3;
        pub const ENV_CALL_FROM_U_OR_VU: usize = 1 << 8;
        pub const INST_PAGE_FAULT: usize = 1 << 12;
        pub const LOAD_PAGE_FAULT: usize = 1 << 13;
        pub const STORE_PAGE_FAULT: usize = 1 << 15;
    }
}
