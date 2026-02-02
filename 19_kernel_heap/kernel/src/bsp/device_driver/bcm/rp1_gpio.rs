// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! RP1 GPIO Driver.

use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    driver,
    exception::asynchronous::IRQNumber,
    memory::{Address, Virtual},
    synchronization::{interface::Mutex, IRQSafeNullLock},
};
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

register_bitfields! {
    u32,

    PADS_CTRL [
        OD OFFSET(7) NUMBITS(1) [],
        IE OFFSET(6) NUMBITS(1) [],
        PUE OFFSET(3) NUMBITS(1) []
    ],

    GPIO_CTRL [
        FUNCSEL OFFSET(0) NUMBITS(5) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub Rp1PadBank {
        (0x00 => _reserved0),
        (0x3C => pub GPIO14: ReadWrite<u32, PADS_CTRL::Register>),
        (0x40 => pub GPIO15: ReadWrite<u32, PADS_CTRL::Register>),
        (0x44 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    pub Rp1GioBank {
        (0x00 => _reserved0),
        (0x74 => pub GPIO14_CTRL: ReadWrite<u32, GPIO_CTRL::Register>),
        (0x78 => _reserved1),
        (0x7C => pub GPIO15_CTRL: ReadWrite<u32, GPIO_CTRL::Register>),
        (0x80 => @END),
    }
}

struct GPIOInner {
    pads: MMIODerefWrapper<Rp1PadBank>,
    gpio: MMIODerefWrapper<Rp1GioBank>,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct GPIO {
    inner: IRQSafeNullLock<GPIOInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl GPIOInner {
    pub const unsafe fn new(pads_addr: Address<Virtual>, gpio_addr: Address<Virtual>) -> Self {
        Self {
            pads: MMIODerefWrapper::new(pads_addr),
            gpio: MMIODerefWrapper::new(gpio_addr),
        }
    }

    pub fn map_pl011_uart(&mut self) {
        // Configure GPIO 14 (TX)
        // OD=0 (Output Disable cleared?), FUNC=4 (UART0)
        self.pads.GPIO14.modify(PADS_CTRL::OD::CLEAR);
        self.gpio.GPIO14_CTRL.write(GPIO_CTRL::FUNCSEL.val(4));

        // Configure GPIO 15 (RX)
        // IE=1 (Input Enable), PUE=1 (Pull Up Enable), FUNC=4
        self.pads
            .GPIO15
            .modify(PADS_CTRL::IE::SET + PADS_CTRL::PUE::SET);
        self.gpio.GPIO15_CTRL.write(GPIO_CTRL::FUNCSEL.val(4));
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl GPIO {
    pub const COMPATIBLE: &'static str = "RP1 GPIO";

    pub const unsafe fn new(pads_addr: Address<Virtual>, gpio_addr: Address<Virtual>) -> Self {
        Self {
            inner: IRQSafeNullLock::new(GPIOInner::new(pads_addr, gpio_addr)),
        }
    }

    pub fn map_pl011_uart(&self) {
        self.inner.lock(|inner| inner.map_pl011_uart())
    }
}

impl driver::interface::DeviceDriver for GPIO {
    type IRQNumberType = IRQNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }
}
