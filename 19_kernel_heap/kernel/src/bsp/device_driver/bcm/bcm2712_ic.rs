// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! BCM2712 Interrupt Controller (MIP + GICv2).

use crate::{
    bsp::device_driver::{arm::gicv2::GICv2, common::MMIODerefWrapper},
    driver, exception,
    memory::{Address, Virtual},
    synchronization::interface::ReadWriteEx,
};
use tock_registers::{
    interfaces::Writeable,
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_structs! {
    #[allow(non_snake_case)]
    pub MipRegs {
        (0x00 => pub MIP_STATUS: ReadOnly<u32>),
        (0x04 => _reserved0),
        (0x20 => pub INT_CFGL_HOST: ReadWrite<u32>),
        (0x24 => _reserved1),
        (0x30 => pub INT_CFGH_HOST: ReadWrite<u32>),
        (0x34 => _reserved2),
        (0x40 => pub INT_MASKL_HOST: ReadWrite<u32>),
        (0x44 => _reserved3),
        (0x50 => pub INT_MASKH_HOST: ReadWrite<u32>),
        (0x54 => _reserved4),
        (0x60 => pub INT_MASKL_VPU: ReadWrite<u32>),
        (0x64 => _reserved5),
        (0x70 => pub INT_MASKH_VPU: ReadWrite<u32>),
        (0x74 => @END),
    }
}

const RP1_APB_OFFSET: usize = 0x8000;
const UART0_VECTOR: usize = 25;

pub struct BCM2712InterruptController {
    gic: GICv2,
    mip: MMIODerefWrapper<MipRegs>,
    rp1_base: Address<Virtual>,
}

impl BCM2712InterruptController {
    pub const COMPATIBLE: &'static str = "BCM2712 IntC";

    pub const unsafe fn new(
        gicd_base: Address<Virtual>,
        gicc_base: Address<Virtual>,
        mip_base: Address<Virtual>,
        rp1_base: Address<Virtual>,
    ) -> Self {
        Self {
            gic: GICv2::new(gicd_base, gicc_base),
            mip: MMIODerefWrapper::new(mip_base),
            rp1_base,
        }
    }

    unsafe fn rearm_rp1_uart(&self) {
        let apb_base = (self.rp1_base + RP1_APB_OFFSET).as_usize() as *mut u32;
        // Re-arm register is at APB_BASE + 0x8 + (Vector * 4)
        let ctrl_reg = apb_base.add(2).add(UART0_VECTOR); // 2 = 0x8/4

        // Write 0xD to re-arm/acknowledge
        core::ptr::write_volatile(ctrl_reg, 0xD);
    }
}

impl driver::interface::DeviceDriver for BCM2712InterruptController {
    type IRQNumberType = exception::asynchronous::IRQNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        // 1. Mask VPU interrupts (Crucial to prevent VPU from stealing IRQs)
        self.mip.INT_MASKL_VPU.set(0xFFFFFFFF);
        self.mip.INT_MASKH_VPU.set(0xFFFFFFFF);

        // 2. Configure Host for Edge Trigger (Active High) to match RP1 signaling
        self.mip.INT_CFGL_HOST.set(0xFFFFFFFF);
        self.mip.INT_CFGH_HOST.set(0xFFFFFFFF);

        // 3. Unmask Host interrupts
        self.mip.INT_MASKL_HOST.set(0);
        self.mip.INT_MASKH_HOST.set(0);

        // 4. Init GIC
        self.gic.init()
    }
}

impl exception::asynchronous::interface::IRQManager for BCM2712InterruptController {
    type IRQNumberType = exception::asynchronous::IRQNumber;

    fn register_handler(
        &self,
        descriptor: exception::asynchronous::IRQHandlerDescriptor<Self::IRQNumberType>,
    ) -> Result<(), &'static str> {
        let irq_num = descriptor.number().get();

        // MIP Input 0 maps to GIC SPI 128 (ID 160).
        // It requires Edge Triggering.
        if irq_num == 160 {
            self.gic.set_trigger(&descriptor.number(), true);
        }

        self.gic.register_handler(descriptor)
    }

    fn enable(&self, irq: &Self::IRQNumberType) {
        self.gic.enable(irq)
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        // Custom handling to support RP1 Re-arm

        // 1. Read IAR
        let irq_number = self.gic.gicc.pending_irq_number(ic);

        // 2. Dispatch
        if irq_number <= 1019 {
            self.gic
                .handler_table
                .read(|table| match table[irq_number] {
                    None => panic!("No handler registered for IRQ {}", irq_number),
                    Some(descriptor) => {
                        descriptor.handler().handle().expect("Error handling IRQ");
                    }
                });

            // 3. Re-arm RP1 if UART (ID 160)
            if irq_number == 160 {
                unsafe {
                    self.rearm_rp1_uart();
                }
            }
        }

        // 4. EOI
        self.gic.gicc.mark_comleted(irq_number as u32, ic);
    }

    fn print_handler(&self) {
        self.gic.print_handler()
    }
}
