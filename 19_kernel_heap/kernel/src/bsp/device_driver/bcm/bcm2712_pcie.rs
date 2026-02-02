// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! PCIe Root Complex & RP1 Southbridge Driver.

use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    driver, exception,
    memory::{Address, Virtual},
};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// PCIe Root Complex Registers
register_bitfields! {
    u32,
    MISC_CTRL [
        SCB_ACCESS_EN OFFSET(12) NUMBITS(1) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub PcieRootComplex {
        (0x0000 => _reserved_padding),
        (0x4008 => pub MISC_CTRL: ReadWrite<u32, MISC_CTRL::Register>),
        (0x400C => _reserved1),
        (0x4034 => pub BAR2_LO: ReadWrite<u32>),
        (0x4038 => pub BAR2_HI: ReadWrite<u32>),
        (0x403C => _reserved2),
        (0x40B4 => pub UBUS_BAR2_LO: ReadWrite<u32>),
        (0x40B8 => pub UBUS_BAR2_HI: ReadWrite<u32>),
        (0x40BC => @END),
    }
}

// RP1 Configuration Space Registers
register_bitfields! {
    u32,
    CMD [
        BUS_MASTER OFFSET(2) NUMBITS(1) [],
        MEM_ACCESS OFFSET(1) NUMBITS(1) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub Rp1Config {
        (0x00 => pub VENDOR_ID: ReadOnly<u16>),
        (0x02 => pub DEVICE_ID: ReadOnly<u16>),
        (0x04 => pub COMMAND: ReadWrite<u32, CMD::Register>),
        (0x08 => _reserved0),
        (0x34 => pub CAP_PTR: ReadOnly<u32>),
        (0x38 => @END),
    }
}

type RcRegisters = MMIODerefWrapper<PcieRootComplex>;
type Rp1Registers = MMIODerefWrapper<Rp1Config>;

// Constants
const UART0_VECTOR: usize = 25;
const MIP_DOORBELL_PCI_LO: u32 = 0xFFFF_F000;
const MIP_DOORBELL_PCI_HI: u32 = 0x0000_00FF;
const MIP_PHYS_ADDR: u64 = 0x10_0013_0000;

// Offsets relative to the RP1_CFG_START mapping (0x1F_0010_0000)
const OFFSET_RP1_CFG: usize = 0x9000;
const OFFSET_MSIX_TABLE: usize = 0x31_0000;
const OFFSET_APB_INTERNAL: usize = 0x8000; // 0x1F_0010_8000 relative to 0x1F_0010_0000

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct BCM2712PCIe {
    rc_regs: RcRegisters,
    rp1_regs: Rp1Registers,
    rp1_mapping_base: Address<Virtual>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl BCM2712PCIe {
    pub const COMPATIBLE: &'static str = "BCM2712 PCIe RC";

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide correct MMIO start addresses.
    pub unsafe fn new(rc_base: Address<Virtual>, rp1_mapping_base: Address<Virtual>) -> Self {
        // RP1 Config is at offset 0x9000 in the mapped region
        let rp1_cfg_addr = rp1_mapping_base + OFFSET_RP1_CFG;

        Self {
            rc_regs: RcRegisters::new(rc_base),
            rp1_regs: Rp1Registers::new(rp1_cfg_addr),
            rp1_mapping_base,
        }
    }

    /// Configure the MSI-X table for a specific vector.
    unsafe fn configure_msix(&self, vector: usize, addr_lo: u32, addr_hi: u32, data: u32) {
        let table_base = (self.rp1_mapping_base + OFFSET_MSIX_TABLE).as_usize() as *mut u32;
        let entry_ptr = table_base.add(vector * 4);

        core::ptr::write_volatile(entry_ptr.add(0), addr_lo);
        core::ptr::write_volatile(entry_ptr.add(1), addr_hi);
        core::ptr::write_volatile(entry_ptr.add(2), data);
        core::ptr::write_volatile(entry_ptr.add(3), 0); // Unmasked
    }

    /// Unmask the interrupt at the RP1 internal APB controller.
    unsafe fn unmask_rp1_irq(&self, vector: usize) {
        let apb_base = (self.rp1_mapping_base + OFFSET_APB_INTERNAL).as_usize() as *mut u32;
        // The interrupt controller registers start at offset 0x8 inside the APB block?
        // Reference: "let ctrl_base = (RP1_APB_BASE + 0x008) as *mut u32;"
        let ctrl_base = apb_base.add(2); // 0x8 / 4 = 2
        let ctrl_reg = ctrl_base.add(vector);

        // 0x9: Bit 3 (Enable) | Bit 0 (Target)
        core::ptr::write_volatile(ctrl_reg, (1 << 3) | (1 << 0));
    }
}

impl driver::interface::DeviceDriver for BCM2712PCIe {
    type IRQNumberType = exception::asynchronous::IRQNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        // 1. Enable System Core Bus (SCB) Access
        self.rc_regs.MISC_CTRL.modify(MISC_CTRL::SCB_ACCESS_EN::SET);

        // 2. Configure BAR2 for Inbound Translation
        // 0x1C = 64-bit | Prefetchable
        self.rc_regs.BAR2_LO.set(MIP_DOORBELL_PCI_LO | 0x1C);
        self.rc_regs.BAR2_HI.set(MIP_DOORBELL_PCI_HI);

        // Map to MIP Physical Address
        self.rc_regs.UBUS_BAR2_LO.set((MIP_PHYS_ADDR as u32) | 1); // Enable
        self.rc_regs.UBUS_BAR2_HI.set((MIP_PHYS_ADDR >> 32) as u32);

        // 3. Enable RP1 Bus Mastering
        self.rp1_regs
            .COMMAND
            .modify(CMD::BUS_MASTER::SET + CMD::MEM_ACCESS::SET);

        // 4. Enable MSI-X Capability on RP1
        // We need to walk the capability list.
        let mut cap_offset = (self.rp1_regs.CAP_PTR.get() & 0xFF) as usize;
        // Start of config space in our virtual mapping
        let base_ptr = (self.rp1_mapping_base + OFFSET_RP1_CFG).as_usize() as *const u8;

        while cap_offset != 0 {
            let cap_hdr_ptr = base_ptr.add(cap_offset) as *const u32;
            let cap_hdr = core::ptr::read_volatile(cap_hdr_ptr);

            if (cap_hdr & 0xFF) == 0x11 {
                // MSI-X ID
                // Bit 31 is Enable
                if (cap_hdr & (1 << 31)) == 0 {
                    core::ptr::write_volatile(cap_hdr_ptr as *mut u32, cap_hdr | (1 << 31));
                }
                break;
            }
            cap_offset = ((cap_hdr >> 8) & 0xFF) as usize;
        }

        // 5. Configure UART0 Interrupt (Vector 25)
        self.configure_msix(UART0_VECTOR, MIP_DOORBELL_PCI_LO, MIP_DOORBELL_PCI_HI, 0);
        self.unmask_rp1_irq(UART0_VECTOR);

        Ok(())
    }
}
