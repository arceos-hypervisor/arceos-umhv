use super::csrs::{RiscvCsrTrait, CSR};
use super::consts::traps;
use super::regs::GprIndex;
use super::vcpu::VCpu;
use tock_registers::LocalRegisterCopy;
use super::csrs::defs::hstatus;
use super::{devices::plic::PlicState, vm_pages::fetch_guest_instruction};
use crate::AxVMHal;
use axaddrspace::{GuestPhysAddr, GuestVirtAddr};
use axerrno::{AxError, AxResult};
use axvcpu::AxArchVCpuExitReason;
use core::panic;
use riscv_decode::Instruction;

/// the devices that belongs to a vm
pub struct DeviceList<H: AxVMHal> {
    plic: PlicState,
    _marker: core::marker::PhantomData<H>,
}

impl<H: AxVMHal> DeviceList<H> {
    /// Create a new VM with `vcpus` vCPUs and `gpt` as the guest page table.
    pub fn new() -> Self {
        DeviceList {
            plic: PlicState::new(0xC00_0000),
            _marker: core::marker::PhantomData,
        }
    }

    pub fn vmexit_handler(
        &mut self,
        vcpu: &mut VCpu<H>,
        vm_exit_info: AxArchVCpuExitReason,
    ) -> AxResult {
        match vm_exit_info {
            AxArchVCpuExitReason::NestedPageFault { addr: fault_addr } => {
                let falut_pc = vcpu.regs().guest_regs.sepc;
                let inst = vcpu.regs().trap_csrs.htinst as u32;
                
                let val = LocalRegisterCopy::<usize, hstatus::Register>::new(vcpu.regs().guest_regs.hstatus);
                
                match val.read(hstatus::spvp) {
                    1 => {
                        match self.handle_page_fault(
                            GuestVirtAddr::from(falut_pc),
                            inst,
                            GuestPhysAddr::from(fault_addr),
                            vcpu,
                        ) {
                            Ok(inst_len) => {
                                vcpu.advance_pc(inst_len);
                            }
                            Err(err) => {
                                panic!(
                                    "Page fault at {:#x} addr@{:#x} with error {:?}",
                                    falut_pc, fault_addr, err
                                )
                            }
                        }
                    }
                    0 => {
                        panic!("User page fault")
                    }
                    _ => unreachable!(), // Field is only 1-bit wide.
                }
            }
            AxArchVCpuExitReason::ExternalInterrupt { .. } => self.handle_irq(),
            _ => {}
        }
        Ok(())
    }
}

// Privaie methods implementation
impl<H: AxVMHal> DeviceList<H> {
    fn handle_page_fault(
        &mut self,
        inst_addr: GuestVirtAddr,
        inst: u32,
        fault_addr: GuestPhysAddr,
        vcpu: &mut VCpu<H>,
    ) -> AxResult<usize> {
        //  plic
        if fault_addr.as_usize() >= self.plic.base()
            && fault_addr.as_usize() < self.plic.base() + 0x0400_0000
        {
            self.handle_plic(inst_addr, inst, fault_addr, vcpu)
        } else {
            error!("inst_addr: {:#x}, fault_addr: {:#x}", inst_addr, fault_addr);
            Err(AxError::BadAddress)
        }
    }

    #[allow(clippy::needless_late_init)]
    fn handle_plic(
        &mut self,
        inst_addr: GuestVirtAddr,
        mut inst: u32,
        fault_addr: GuestPhysAddr,
        vcpu: &mut VCpu<H>,
    ) -> AxResult<usize> {
        if inst == 0 {
            // If hinst does not provide information about trap,
            // we must read the instruction from guest's memory maunally.
            inst = fetch_guest_instruction(inst_addr)?;
        }
        let i1 = inst as u16;
        let len = riscv_decode::instruction_length(i1);
        let inst = match len {
            2 => i1 as u32,
            4 => inst,
            _ => unreachable!(),
        };
        // assert!(len == 4);
        let decode_inst = riscv_decode::decode(inst).map_err(|_| AxError::InvalidData)?;
        match decode_inst {
            Instruction::Sw(i) => {
                let val = vcpu.get_gpr(GprIndex::from_raw(i.rs2()).unwrap()) as u32;
                self.plic.write_u32(fault_addr.as_usize(), val)
            }
            Instruction::Lw(i) => {
                let val = self.plic.read_u32(fault_addr.as_usize());
                vcpu.set_gpr(GprIndex::from_raw(i.rd()).unwrap(), val as usize)
            }
            _ => return Err(AxError::BadAddress),
        }
        Ok(len)
    }

    fn handle_irq(&mut self) {
        let context_id = 1;
        let claim_and_complete_addr = self.plic.base() + 0x0020_0004 + 0x1000 * context_id;
        let irq = unsafe { core::ptr::read_volatile(claim_and_complete_addr as *const u32) };
        assert!(irq != 0);
        self.plic.claim_complete[context_id] = irq;

        CSR.hvip
            .read_and_set_bits(traps::interrupt::VIRTUAL_SUPERVISOR_EXTERNAL);
    }
}
