/// Number of contexts for the PLIC. Value is twice the max number of harts because each hart will
/// have one M-mode context and one S-mode context.
pub const MAX_CONTEXTS: usize = 15872;
pub const MAX_DEVICES: usize = 1024;

pub const PRIORITY_PER_ID: usize = 4;
pub const PRIORITY_BASE: usize = 0;

pub const ENABLE_BASE: usize = 0x2000;
pub const ENABLE_PER_HART: usize = 0x80;

pub const CONTEXT_BASE: usize = 0x200000;
pub const CONTEXT_PER_HART: usize = 0x1000;
pub const CONTEXT_THRESHOLD: usize = 0;
pub const CONTEXT_CLAIM: usize = 4;

pub const REG_SIZE: usize = 0x1000000;

struct PlicContext {
	/* Static Configuration */
	num: u32,

	/* Local IRQ state */
	irq_priority_threshold: u8,
	irq_enable: [u32; MAX_DEVICES/32],
	irq_pending: [u32; MAX_DEVICES/32],
	irq_pending_priority: [u8; MAX_DEVICES],
	irq_claimed: [u32; MAX_DEVICES/32],
}

impl PlicContext {
    pub fn new(num: u32) -> Self {
        Self {
            num: num,
            irq_priority_threshold: 0,
            irq_enable: [0; MAX_DEVICES/32],
	        irq_pending: [0; MAX_DEVICES/32],
	        irq_pending_priority: [0; MAX_DEVICES],
	        irq_claimed: [0; MAX_DEVICES/32],
        }
    }
}

struct PlicState {
    base_addr: usize,
    // TODO: vm

	/* Static Configuration */
	base_irq: u32,
	num_irq: u32,
	num_irq_word: u32,
	max_prio: u32,
	parent_irq: u32,

	/* Context Array */
	num_context: u32,
	contexts: Vec<PlicContext>,

	/* Global IRQ state */
	irq_priority: [u8; MAX_DEVICES],
	irq_level: [u32; MAX_DEVICES/32],
}


impl PlicState {
    pub fn new(base_addr: usize, vcpu_count: usize) -> Self {
        
        let mut contexts = Vec::with_capacity(vcpu_count * 2);

        for i in 0..vcpu_count*2 {
            contexts.push(PlicContext::new(i));
        }
        

        Self {
            base_addr: base,
            // these should be read from config 
            base_irq: 0,
            num_irq: MAX_DEVICES,
            num_irq_word: MAX_DEVICES/32,
            max_prio: 1UL << PRIORITY_PER_ID,
            parent_irq: 0,

            num_context: vcpu_count * 2,
            contexts: contexts,

            irq_priority: [0; MAX_DEVICES],
	        irq_level: [0; MAX_DEVICES/32],
        }
    }

    pub fn plic_emulator_read(&mut self, offset: usize, size: u32)->u32 {
        let mut dst: u32 = 0;
        offset &= !0x3;

        if PRIORITY_BASE <= offset && offset < ENABLE_BASE {
            dst = self.plic_priority_read(offset);
        } else if ENABLE_BASE <= offset && offset < CONTEXT_BASE {
            cntx = (offset - ENABLE_BASE) / ENABLE_PER_HART;
            offset -= cntx * ENABLE_PER_HART + ENABLE_BASE;
            if (cntx < self.num_context) {
                dst = self.plic_context_enable_read(offset, cntx);
            }
        } else if (CONTEXT_BASE <= offset && offset < REG_SIZE) {
            cntx = (offset - CONTEXT_BASE) / CONTEXT_PER_HART;
            offset -= cntx * CONTEXT_PER_HART + CONTEXT_BASE;
            if (cntx < self.num_context) {
                dst = plic_context_read(offset, cntx);
            }
        }
        
        dst;
    }

    pub fn plic_emulator_write(&mut self, offset: usize, size: u32) {

    }
}

impl PlicState {
    fn plic_priority_read(self, offset: usize) -> u32{
        let irq = (offset >> 2);

        // TODO: add return error
        // if (irq == 0 || irq >= self.num_irq) {
        //     return VMM_EINVALID;
        // }

	    let dst = self.irq_priority[irq];
        dst
    }

    fn plic_priority_write(&mut self, offset: usize, src_mask: u32, src: u32) {
        let irq: u32 = (offset >> 2);

        // if (irq == 0 || irq >= self.num_irq) {
        // return VMM_EINVALID;
        // }

        val = self.irq_priority[irq] & src_mask;
        val |= src & !src_mask;
        val &= ((1 << PRIORITY_PER_ID) - 1);
        self.irq_priority[irq] = val;
    }

    fn plic_context_enable_read(self, offset: usize, cntx: u32) -> u32{
        let irq_word = offset >> 2;

        if self.num_irq_word < irq_word {
            return VMM_EINVALID;
        }
        let dst = self.contexts[cntx].irq_enable[irq_word];

        dst
    }
    
    fn plic_context_enable_write(&mut self, offset: usize, cntx: u32, src_mask: u32, src: u32) -> u32{

        let irq_word:u32 = offset >> 2;

        if self.num_irq_word < irq_word {
            return VMM_EINVALID;
        }
        let old_val = self.contexts[cntx].irq_enable[irq_word];
        let new_val = (old_val & src_mask) | (src & !src_mask);
        if (irq_word == 0) {
            new_val &= !0x1;
        }
        self.contexts[cntx].irq_enable[irq_word] = new_val;

        let xor_val = old_val ^ new_val;
        for i in 0..32 {
            irq = irq_word * 32 + i;
            irq_mask = 1 << i;
            irq_prio = self.irq_priority[irq];
            if !(xor_val & irq_mask) {
                continue;
            }
                
            if (new_val & irq_mask) &&
                (self.irq_level[irq_word] & irq_mask) {
                self.contexts[cntx].irq_pending[irq_word] |= irq_mask;
                self.contexts[cntx].irq_pending_priority[irq] = irq_prio;
            } else if !(new_val & irq_mask) {
                self.contexts[cntx].irq_pending[irq_word] &= !irq_mask;
                self.contexts[cntx].irq_pending_priority[irq] = 0;
                self.contexts[cntx].irq_claimed[irq_word] &= !irq_mask;
            }
        }

        __plic_context_irq_update(s, c);

        return VMM_OK;
    }

    fn plic_context_read(self, offset: usize, cntx: u32) -> u32{
        match offset {
            CONTEXT_THRESHOLD => {
                return self.contexts[cntx].irq_priority_threshold;
            }
            CONTEXT_CLAIM => {
                return __plic_context_irq_claim(cntx);
            }
            _ => {
                return 0;
            }
        }
    }

    /* Note: Must be called with c->irq_lock held */
    fn __plic_context_best_pending_irq(self, cntx: u32) -> u32 {
        let best_irq_prio:u8 = 0;
        let mut best_irq:u32 = 0;

        for i in 0..self.num_irq_word {
            if (!self.contexts[cntx].irq_pending[i]) {
                continue;
            }

            for j in 0..32 {
                let irq = i * 32 + j;
                if ((self.num_irq <= irq) ||
                    !(self.contexts[cntx].irq_pending[i] & (1 << j)) ||
                    (self.contexts[cntx].irq_claimed[i] & (1 << j))) {
                    continue;
                }
                if (!best_irq ||(best_irq_prio < self.contexts[cntx].irq_pending_priority[irq])) {
                    best_irq = irq;
                    best_irq_prio = self.contexts[cntx].irq_pending_priority[irq];
                }
            }
        }

        return best_irq;
    }

}