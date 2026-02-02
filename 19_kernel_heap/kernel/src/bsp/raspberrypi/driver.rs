// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

//! BSP driver support.

use super::{exception, memory::map::mmio};
use crate::{
    bsp::device_driver,
    console, driver as generic_driver,
    exception::{self as generic_exception},
    memory,
    memory::mmu::MMIODescriptor,
};

#[cfg(feature = "bsp_rpi5")]
use crate::driver::interface::DeviceDriver;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static mut PL011_UART: MaybeUninit<device_driver::PL011Uart> = MaybeUninit::uninit();
static mut GPIO: MaybeUninit<device_driver::GPIO> = MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi3")]
static mut INTERRUPT_CONTROLLER: MaybeUninit<device_driver::InterruptController> =
    MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi4")]
static mut INTERRUPT_CONTROLLER: MaybeUninit<device_driver::GICv2> = MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi5")]
static mut INTERRUPT_CONTROLLER: MaybeUninit<device_driver::BCM2712InterruptController> =
    MaybeUninit::uninit();

#[cfg(feature = "bsp_rpi5")]
static mut PCIE: MaybeUninit<device_driver::BCM2712PCIe> = MaybeUninit::uninit();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// This must be called only after successful init of the memory subsystem.
unsafe fn instantiate_uart() -> Result<(), &'static str> {
    let mmio_descriptor = MMIODescriptor::new(mmio::PL011_UART_START, mmio::PL011_UART_SIZE);
    let virt_addr =
        memory::mmu::kernel_map_mmio(device_driver::PL011Uart::COMPATIBLE, &mmio_descriptor)?;

    PL011_UART.write(device_driver::PL011Uart::new(virt_addr));

    Ok(())
}

/// This must be called only after successful init of the UART driver.
unsafe fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(PL011_UART.assume_init_ref());

    Ok(())
}

/// This must be called only after successful init of the memory subsystem.
#[cfg(not(feature = "bsp_rpi5"))]
unsafe fn instantiate_gpio() -> Result<(), &'static str> {
    let mmio_descriptor = MMIODescriptor::new(mmio::GPIO_START, mmio::GPIO_SIZE);
    let virt_addr =
        memory::mmu::kernel_map_mmio(device_driver::GPIO::COMPATIBLE, &mmio_descriptor)?;

    GPIO.write(device_driver::GPIO::new(virt_addr));

    Ok(())
}

#[cfg(feature = "bsp_rpi5")]
unsafe fn instantiate_gpio() -> Result<(), &'static str> {
    let gpio_desc = MMIODescriptor::new(mmio::GPIO_START, mmio::GPIO_SIZE);
    let gpio_virt = memory::mmu::kernel_map_mmio("RP1 GPIO", &gpio_desc)?;

    let pads_desc = MMIODescriptor::new(mmio::PADS_START, mmio::PADS_SIZE);
    let pads_virt = memory::mmu::kernel_map_mmio("RP1 PADS", &pads_desc)?;

    GPIO.write(device_driver::GPIO::new(pads_virt, gpio_virt));

    Ok(())
}

/// This must be called only after successful init of the GPIO driver.
unsafe fn post_init_gpio() -> Result<(), &'static str> {
    GPIO.assume_init_ref().map_pl011_uart();
    Ok(())
}

/// This must be called only after successful init of the memory subsystem.
#[cfg(feature = "bsp_rpi3")]
unsafe fn instantiate_interrupt_controller() -> Result<(), &'static str> {
    let periph_mmio_descriptor =
        MMIODescriptor::new(mmio::PERIPHERAL_IC_START, mmio::PERIPHERAL_IC_SIZE);
    let periph_virt_addr = memory::mmu::kernel_map_mmio(
        device_driver::InterruptController::COMPATIBLE,
        &periph_mmio_descriptor,
    )?;

    INTERRUPT_CONTROLLER.write(device_driver::InterruptController::new(periph_virt_addr));

    Ok(())
}

/// This must be called only after successful init of the memory subsystem.
#[cfg(feature = "bsp_rpi4")]
unsafe fn instantiate_interrupt_controller() -> Result<(), &'static str> {
    let gicd_mmio_descriptor = MMIODescriptor::new(mmio::GICD_START, mmio::GICD_SIZE);
    let gicd_virt_addr = memory::mmu::kernel_map_mmio("GICv2 GICD", &gicd_mmio_descriptor)?;

    let gicc_mmio_descriptor = MMIODescriptor::new(mmio::GICC_START, mmio::GICC_SIZE);
    let gicc_virt_addr = memory::mmu::kernel_map_mmio("GICV2 GICC", &gicc_mmio_descriptor)?;

    INTERRUPT_CONTROLLER.write(device_driver::GICv2::new(gicd_virt_addr, gicc_virt_addr));

    Ok(())
}

#[cfg(feature = "bsp_rpi5")]
unsafe fn instantiate_interrupt_controller() -> Result<(), &'static str> {
    let gicd_desc = MMIODescriptor::new(mmio::GICD_START, mmio::GICD_SIZE);
    let gicd_virt = memory::mmu::kernel_map_mmio("GICv2 GICD", &gicd_desc)?;

    let gicc_desc = MMIODescriptor::new(mmio::GICC_START, mmio::GICC_SIZE);
    let gicc_virt = memory::mmu::kernel_map_mmio("GICv2 GICC", &gicc_desc)?;

    let mip_desc = MMIODescriptor::new(mmio::MIP_START, mmio::MIP_SIZE);
    let mip_virt = memory::mmu::kernel_map_mmio("MIP", &mip_desc)?;

    let rp1_desc = MMIODescriptor::new(mmio::RP1_CFG_START, mmio::RP1_CFG_SIZE);
    let rp1_virt = memory::mmu::kernel_map_mmio("RP1 Config (IntC)", &rp1_desc)?;

    INTERRUPT_CONTROLLER.write(device_driver::BCM2712InterruptController::new(
        gicd_virt, gicc_virt, mip_virt, rp1_virt,
    ));

    Ok(())
}

#[cfg(feature = "bsp_rpi5")]
unsafe fn instantiate_pcie() -> Result<(), &'static str> {
    let rc_desc = MMIODescriptor::new(mmio::PCIE_RC_START, mmio::PCIE_RC_SIZE);
    let rc_virt = memory::mmu::kernel_map_mmio("PCIe RC", &rc_desc)?;

    let rp1_desc = MMIODescriptor::new(mmio::RP1_CFG_START, mmio::RP1_CFG_SIZE);
    let rp1_virt = memory::mmu::kernel_map_mmio("RP1 Config", &rp1_desc)?;

    PCIE.write(device_driver::BCM2712PCIe::new(rc_virt, rp1_virt));

    Ok(())
}

#[cfg(feature = "bsp_rpi5")]
unsafe fn driver_pcie() -> Result<(), &'static str> {
    instantiate_pcie()?;
    PCIE.assume_init_ref().init()
}

/// This must be called only after successful init of the interrupt controller driver.
unsafe fn post_init_interrupt_controller() -> Result<(), &'static str> {
    generic_exception::asynchronous::register_irq_manager(INTERRUPT_CONTROLLER.assume_init_ref());

    Ok(())
}

/// Function needs to ensure that driver registration happens only after correct instantiation.
unsafe fn driver_uart() -> Result<(), &'static str> {
    instantiate_uart()?;

    let uart_descriptor = generic_driver::DeviceDriverDescriptor::new(
        PL011_UART.assume_init_ref(),
        Some(post_init_uart),
        Some(exception::asynchronous::irq_map::PL011_UART),
    );
    generic_driver::driver_manager().register_driver(uart_descriptor);

    Ok(())
}

/// Function needs to ensure that driver registration happens only after correct instantiation.
unsafe fn driver_gpio() -> Result<(), &'static str> {
    instantiate_gpio()?;

    let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(
        GPIO.assume_init_ref(),
        Some(post_init_gpio),
        None,
    );
    generic_driver::driver_manager().register_driver(gpio_descriptor);

    Ok(())
}

/// Function needs to ensure that driver registration happens only after correct instantiation.
unsafe fn driver_interrupt_controller() -> Result<(), &'static str> {
    instantiate_interrupt_controller()?;

    let interrupt_controller_descriptor = generic_driver::DeviceDriverDescriptor::new(
        INTERRUPT_CONTROLLER.assume_init_ref(),
        Some(post_init_interrupt_controller),
        None,
    );
    generic_driver::driver_manager().register_driver(interrupt_controller_descriptor);

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Initialize the driver subsystem.
///
/// # Safety
///
/// See child function calls.
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    #[cfg(feature = "bsp_rpi5")]
    driver_pcie()?;

    driver_uart()?;
    driver_gpio()?;
    driver_interrupt_controller()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}

/// Minimal code needed to bring up the console in QEMU (for testing only). This is often less steps
/// than on real hardware due to QEMU's abstractions.
#[cfg(feature = "test_build")]
pub fn qemu_bring_up_console() {
    use crate::cpu;

    unsafe {
        instantiate_uart().unwrap_or_else(|_| cpu::qemu_exit_failure());
        console::register_console(PL011_UART.assume_init_ref());
    };
}
