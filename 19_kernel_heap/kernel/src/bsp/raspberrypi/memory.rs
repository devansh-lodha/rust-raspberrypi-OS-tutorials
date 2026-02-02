// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! BSP Memory Management.
//!
//! The physical memory layout.
//!
//! The Raspberry's firmware copies the kernel binary to 0x8_0000. The preceding region will be used
//! as the boot core's stack.
//!
//! +---------------------------------------+
//! |                                       | boot_core_stack_start @ 0x0
//! |                                       |                                ^
//! | Boot-core Stack                       |                                | stack
//! |                                       |                                | growth
//! |                                       |                                | direction
//! +---------------------------------------+
//! |                                       | code_start @ 0x8_0000 == boot_core_stack_end_exclusive
//! | .text                                 |
//! | .rodata                               |
//! | .got                                  |
//! | .kernel_symbols                       |
//! |                                       |
//! +---------------------------------------+
//! |                                       | data_start == code_end_exclusive
//! | .data                                 |
//! | .bss                                  |
//! |                                       |
//! +---------------------------------------+
//! |                                       | heap_start == data_end_exclusive
//! | .heap                                 |
//! |                                       |
//! +---------------------------------------+
//! |                                       | heap_end_exclusive
//! |                                       |
//!
//!
//!
//!
//!
//! The virtual memory layout is as follows:
//!
//! +---------------------------------------+
//! |                                       | code_start @ __kernel_virt_start_addr
//! | .text                                 |
//! | .rodata                               |
//! | .got                                  |
//! | .kernel_symbols                       |
//! |                                       |
//! +---------------------------------------+
//! |                                       | data_start == code_end_exclusive
//! | .data                                 |
//! | .bss                                  |
//! |                                       |
//! +---------------------------------------+
//! |                                       | heap_start == data_end_exclusive
//! | .heap                                 |
//! |                                       |
//! +---------------------------------------+
//! |                                       |  mmio_remap_start == heap_end_exclusive
//! | VA region for MMIO remapping          |
//! |                                       |
//! +---------------------------------------+
//! |                                       |  mmio_remap_end_exclusive
//! | Unmapped guard page                   |
//! |                                       |
//! +---------------------------------------+
//! |                                       | boot_core_stack_start
//! |                                       |                                ^
//! | Boot-core Stack                       |                                | stack
//! |                                       |                                | growth
//! |                                       |                                | direction
//! +---------------------------------------+
//! |                                       | boot_core_stack_end_exclusive
//! |                                       |
pub mod mmu;

use crate::memory::{mmu::PageAddress, Address, Physical, Virtual};
use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbols from the linker script.
extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end_exclusive: UnsafeCell<()>;

    static __data_start: UnsafeCell<()>;
    static __data_end_exclusive: UnsafeCell<()>;

    static __heap_start: UnsafeCell<()>;
    static __heap_end_exclusive: UnsafeCell<()>;

    static __mmio_remap_start: UnsafeCell<()>;
    static __mmio_remap_end_exclusive: UnsafeCell<()>;

    static __boot_core_stack_start: UnsafeCell<()>;
    static __boot_core_stack_end_exclusive: UnsafeCell<()>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's physical memory map.
#[rustfmt::skip]
pub(super) mod map {
    use super::*;

    /// Physical devices.
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const PERIPHERAL_IC_START: Address<Physical> = Address::new(0x3F00_B200);
        pub const PERIPHERAL_IC_SIZE:  usize             =              0x24;

        pub const GPIO_START:          Address<Physical> = Address::new(0x3F20_0000);
        pub const GPIO_SIZE:           usize             =              0xA0;

        pub const PL011_UART_START:    Address<Physical> = Address::new(0x3F20_1000);
        pub const PL011_UART_SIZE:     usize             =              0x48;

        pub const END:                 Address<Physical> = Address::new(0x4001_0000);
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const GPIO_START:       Address<Physical> = Address::new(0xFE20_0000);
        pub const GPIO_SIZE:        usize             =              0xA0;

        pub const PL011_UART_START: Address<Physical> = Address::new(0xFE20_1000);
        pub const PL011_UART_SIZE:  usize             =              0x48;

        pub const GICD_START:       Address<Physical> = Address::new(0xFF84_1000);
        pub const GICD_SIZE:        usize             =              0x824;

        pub const GICC_START:       Address<Physical> = Address::new(0xFF84_2000);
        pub const GICC_SIZE:        usize             =              0x14;

        pub const END:              Address<Physical> = Address::new(0xFF85_0000);
    }

    /// Physical devices.
    #[cfg(feature = "bsp_rpi5")]
    pub mod mmio {
        use super::*;

        // GICv2 on BCM2712 (Northbridge)
        pub const GICD_START:      Address<Physical> = Address::new(0x10_7FFF_9000);
        pub const GICD_SIZE:       usize             =              0x1000;
        pub const GICC_START:      Address<Physical> = Address::new(0x10_7FFF_A000);
        pub const GICC_SIZE:       usize             =              0x1000;

        // MIP (Machine Interrupt Peripheral)
        pub const MIP_START:       Address<Physical> = Address::new(0x10_0013_0000);
        pub const MIP_SIZE:        usize             =              0x1000;

        // PCIe Root Complex
        pub const PCIE_RC_START:   Address<Physical> = Address::new(0x10_0012_0000);
        pub const PCIE_RC_SIZE:    usize             =              0x1000;

        // RP1 Southbridge Config Space (ECAM)
        // 0x1F_0010_0000 -> 0x1F_0050_0000 (4MB) covers Config Space + MSI-X Table
        pub const RP1_CFG_START:   Address<Physical> = Address::new(0x1F_0010_0000);
        pub const RP1_CFG_SIZE:    usize             =              0x40_0000;

        // RP1 Peripherals
        // UART is at offset 0x30000 in the RP1 peripheral bar
        pub const PL011_UART_START: Address<Physical> = Address::new(0x1F_0003_0000);
        pub const PL011_UART_SIZE:  usize             =              0x1000;

        pub const GPIO_START:       Address<Physical> = Address::new(0x1F_000D_0000);
        pub const GPIO_SIZE:        usize             =              0x1000;

        pub const PADS_START:       Address<Physical> = Address::new(0x1F_000F_0000);
        pub const PADS_SIZE:        usize             =              0x1000;

        // Used by Translation Table Tool
        pub const END:             Address<Physical> = Address::new(0x20_0000_0000);
    }

    pub const END: Address<Physical> = mmio::END;
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// Start page address of the code segment.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn virt_code_start() -> PageAddress<Virtual> {
    PageAddress::from(unsafe { __code_start.get() as usize })
}

/// Size of the code segment.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn code_size() -> usize {
    unsafe { (__code_end_exclusive.get() as usize) - (__code_start.get() as usize) }
}

/// Start page address of the data segment.
#[inline(always)]
fn virt_data_start() -> PageAddress<Virtual> {
    PageAddress::from(unsafe { __data_start.get() as usize })
}

/// Size of the data segment.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn data_size() -> usize {
    unsafe { (__data_end_exclusive.get() as usize) - (__data_start.get() as usize) }
}

/// Start page address of the heap segment.
#[inline(always)]
fn virt_heap_start() -> PageAddress<Virtual> {
    PageAddress::from(unsafe { __heap_start.get() as usize })
}

/// Size of the heap segment.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn heap_size() -> usize {
    unsafe { (__heap_end_exclusive.get() as usize) - (__heap_start.get() as usize) }
}

/// Start page address of the MMIO remap reservation.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn virt_mmio_remap_start() -> PageAddress<Virtual> {
    PageAddress::from(unsafe { __mmio_remap_start.get() as usize })
}

/// Size of the MMIO remap reservation.
///
/// # Safety
///
/// - Value is provided by the linker script and must be trusted as-is.
#[inline(always)]
fn mmio_remap_size() -> usize {
    unsafe { (__mmio_remap_end_exclusive.get() as usize) - (__mmio_remap_start.get() as usize) }
}

/// Start page address of the boot core's stack.
#[inline(always)]
fn virt_boot_core_stack_start() -> PageAddress<Virtual> {
    PageAddress::from(unsafe { __boot_core_stack_start.get() as usize })
}

/// Size of the boot core's stack.
#[inline(always)]
fn boot_core_stack_size() -> usize {
    unsafe {
        (__boot_core_stack_end_exclusive.get() as usize) - (__boot_core_stack_start.get() as usize)
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Exclusive end address of the physical address space.
#[inline(always)]
pub fn phys_addr_space_end_exclusive_addr() -> PageAddress<Physical> {
    PageAddress::from(map::END)
}
