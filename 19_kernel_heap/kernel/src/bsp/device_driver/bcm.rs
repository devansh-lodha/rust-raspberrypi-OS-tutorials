// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! BCM driver top level.

#[cfg(not(feature = "bsp_rpi5"))]
mod bcm2xxx_gpio;
#[cfg(feature = "bsp_rpi3")]
mod bcm2xxx_interrupt_controller;
mod bcm2xxx_pl011_uart;

#[cfg(not(feature = "bsp_rpi5"))]
pub use bcm2xxx_gpio::*;
#[cfg(feature = "bsp_rpi3")]
pub use bcm2xxx_interrupt_controller::*;
pub use bcm2xxx_pl011_uart::*;

#[cfg(feature = "bsp_rpi5")]
mod rp1_gpio;
#[cfg(feature = "bsp_rpi5")]
pub use rp1_gpio::*;

#[cfg(feature = "bsp_rpi5")]
mod bcm2712_ic;
#[cfg(feature = "bsp_rpi5")]
pub use bcm2712_ic::*;

#[cfg(feature = "bsp_rpi5")]
mod bcm2712_pcie;
#[cfg(feature = "bsp_rpi5")]
pub use bcm2712_pcie::*;
