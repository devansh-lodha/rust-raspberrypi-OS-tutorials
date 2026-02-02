## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
## Copyright (c) 2026 Devansh Lodha <devanshlodha12@gmail.com>

include ../common/docker.mk
include ../common/format.mk
include ../common/operating_system.mk

##--------------------------------------------------------------------------------------------------
## Optional, user-provided configuration values
##--------------------------------------------------------------------------------------------------

# Default to the RPi3.
BSP ?= rpi3

# Default to a serial device name that is common in Linux.
DEV_SERIAL ?= /dev/ttyUSB0

# Optional debug prints.
ifdef DEBUG_PRINTS
    FEATURES = --features debug_prints
endif

# Optional integration test name.
ifdef TEST
    TEST_ARG = --test $(TEST)
else
    TEST_ARG = --test '*'
endif



##--------------------------------------------------------------------------------------------------
## BSP-specific configuration values
##--------------------------------------------------------------------------------------------------
QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

ifeq ($(BSP),rpi3)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE = raspi3
    QEMU_RELEASE_ARGS = -serial stdio -display none
    QEMU_TEST_ARGS    = $(QEMU_RELEASE_ARGS) -semihosting
    OBJDUMP_BINARY    = aarch64-none-elf-objdump
    NM_BINARY         = aarch64-none-elf-nm
    READELF_BINARY    = aarch64-none-elf-readelf
    OPENOCD_ARG       = -f /openocd/tcl/interface/ftdi/olimex-arm-usb-tiny-h.cfg -f /openocd/rpi3.cfg
    JTAG_BOOT_IMAGE   = ../X1_JTAG_boot/jtag_boot_rpi3.img
    LD_SCRIPT_PATH    = $(shell pwd)/kernel/src/bsp/raspberrypi
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 -C force-frame-pointers
else ifeq ($(BSP),rpi4)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE =
    QEMU_RELEASE_ARGS = -serial stdio -display none
    QEMU_TEST_ARGS    = $(QEMU_RELEASE_ARGS) -semihosting
    OBJDUMP_BINARY    = aarch64-none-elf-objdump
    NM_BINARY         = aarch64-none-elf-nm
    READELF_BINARY    = aarch64-none-elf-readelf
    OPENOCD_ARG       = -f /openocd/tcl/interface/ftdi/olimex-arm-usb-tiny-h.cfg -f /openocd/rpi4.cfg
    JTAG_BOOT_IMAGE   = ../X1_JTAG_boot/jtag_boot_rpi4.img
    LD_SCRIPT_PATH    = $(shell pwd)/kernel/src/bsp/raspberrypi
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72 -C force-frame-pointers
else ifeq ($(BSP),rpi5)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    QEMU_BINARY       = echo "RPi5 QEMU not supported"; \#
    QEMU_MACHINE_TYPE =
    QEMU_RELEASE_ARGS =
    QEMU_TEST_ARGS    =
    OBJDUMP_BINARY    = aarch64-none-elf-objdump
    NM_BINARY         = aarch64-none-elf-nm
    READELF_BINARY    = aarch64-none-elf-readelf
    GDB_BINARY        = aarch64-elf-gdb
    GDB_INIT_FILE     = ../debug/pi5/gdb-init.txt
    OPENOCD_ARG       = -f ../debug/pi5/cmsis-dap.cfg -f ../debug/pi5/raspberrypi5.cfg
    JTAG_BOOT_IMAGE   = ../X2_pi5_jtag_halt_stub/halt_stub.img
    LD_SCRIPT_PATH    = $(shell pwd)/kernel/src/bsp/raspberrypi
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a76 -C force-frame-pointers
endif

# Export for build.rs.
export LD_SCRIPT_PATH



##--------------------------------------------------------------------------------------------------
## Targets and Prerequisites
##--------------------------------------------------------------------------------------------------
KERNEL_MANIFEST      = kernel/Cargo.toml
KERNEL_LINKER_SCRIPT = kernel.ld
LAST_BUILD_CONFIG    = target/$(BSP)_$(DEBUG_PRINTS).build_config

KERNEL_ELF_RAW      = target/$(TARGET)/release/kernel
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
KERNEL_ELF_RAW_DEPS = $(filter-out %: ,$(file < $(KERNEL_ELF_RAW).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)

##------------------------------------------------------------------------------
## Translation tables
##------------------------------------------------------------------------------
TT_TOOL_PATH = tools/translation_table_tool

KERNEL_ELF_TTABLES      = target/$(TARGET)/release/kernel+ttables
KERNEL_ELF_TTABLES_DEPS = $(KERNEL_ELF_RAW) $(wildcard $(TT_TOOL_PATH)/*)

##------------------------------------------------------------------------------
## Kernel symbols
##------------------------------------------------------------------------------
export KERNEL_SYMBOLS_TOOL_PATH = tools/kernel_symbols_tool

KERNEL_ELF_TTABLES_SYMS = target/$(TARGET)/release/kernel+ttables+symbols

# Unlike with KERNEL_ELF_RAW, we are not relying on dep-info here. One of the reasons being that the
# name of the generated symbols file varies between runs, which can cause confusion.
KERNEL_ELF_TTABLES_SYMS_DEPS = $(KERNEL_ELF_TTABLES) \
    $(wildcard kernel_symbols/*)                     \
    $(wildcard $(KERNEL_SYMBOLS_TOOL_PATH)/*)

export TARGET
export KERNEL_SYMBOLS_INPUT_ELF  = $(KERNEL_ELF_TTABLES)
export KERNEL_SYMBOLS_OUTPUT_ELF = $(KERNEL_ELF_TTABLES_SYMS)

KERNEL_ELF = $(KERNEL_ELF_TTABLES_SYMS)



##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
RUSTFLAGS = $(RUSTC_MISC_ARGS)                   \
    -C link-arg=--library-path=$(LD_SCRIPT_PATH) \
    -C link-arg=--script=$(KERNEL_LINKER_SCRIPT)

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
    -D warnings                   \
    -D missing_docs

FEATURES     += --features bsp_$(BSP)
COMPILER_ARGS = --target=$(TARGET) \
    $(FEATURES)                    \
    --release

# build-std can be skipped for helper commands that do not rely on correct stack frames and other
# custom compiler options. This results in a huge speedup.
BASE_RUSTC_CMD   = cargo rustc $(COMPILER_ARGS) -Z build-std=core,alloc --manifest-path $(KERNEL_MANIFEST)
BASE_DOC_CMD     = cargo doc $(COMPILER_ARGS)
BASE_CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
BASE_TEST_CMD    = cargo test $(COMPILER_ARGS) -Z build-std=core,alloc --manifest-path $(KERNEL_MANIFEST)
BASE_OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
EXEC_TT_TOOL       = ruby $(TT_TOOL_PATH)/main.rb
EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
EXEC_MINIPUSH      = ruby ../common/serial/minipush.rb

##------------------------------------------------------------------------------
## Dockerization
##------------------------------------------------------------------------------
DOCKER_CMD            = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
DOCKER_CMD_INTERACT   = $(DOCKER_CMD) -i
DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/../common:/work/common
DOCKER_ARG_DIR_JTAG   = -v $(shell pwd)/../X1_JTAG_boot:/work/X1_JTAG_boot
DOCKER_ARG_DIR_HALT   = -v $(shell pwd)/../X2_pi5_jtag_halt_stub:/work/X2_pi5_jtag_halt_stub
DOCKER_ARG_DIR_DEBUG  = -v $(shell pwd)/../debug:/work/debug
DOCKER_ARG_DEV        = --privileged -v /dev:/dev
DOCKER_ARG_NET        = --network host

# DOCKER_IMAGE defined in include file (see top of this file).
DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
DOCKER_GDB   = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_NET) $(DOCKER_ARG_DIR_DEBUG) $(DOCKER_IMAGE)

# Dockerize commands, which require USB device passthrough, only on Linux.
ifeq ($(shell uname -s),Linux)
    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
    DOCKER_JTAGBOOT  = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_ARG_DIR_JTAG) $(DOCKER_IMAGE)
    DOCKER_OPENOCD   = $(DOCKER_CMD_DEV) $(DOCKER_ARG_NET) $(DOCKER_ARG_DIR_DEBUG) $(DOCKER_IMAGE)
    STUB_MAKE_CMD    = $(DOCKER_CMD) $(DOCKER_ARG_DIR_HALT) -w /work/X2_pi5_jtag_halt_stub $(DOCKER_IMAGE) make
else ifeq ($(shell uname -s),Darwin)
    DOCKER_OPENOCD   =
    GDB_CMD          = $(GDB_BINARY)
    STUB_MAKE_CMD    = $(MAKE) -C ../X2_pi5_jtag_halt_stub
else
    DOCKER_OPENOCD   = echo "Not yet supported on non-Linux systems."; \#
    STUB_MAKE_CMD    = echo "Not yet supported on non-Linux systems."; \#
endif

# These commands are always local, as per the repository's design
RUSTC_CMD   = $(BASE_RUSTC_CMD)
DOC_CMD     = $(BASE_DOC_CMD)
CLIPPY_CMD  = $(BASE_CLIPPY_CMD)
TEST_CMD    = $(BASE_TEST_CMD)
OBJCOPY_CMD = $(BASE_OBJCOPY_CMD)
CLEAN_CMD   = cargo clean

##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------
.PHONY: all doc qemu chainboot clippy clean readelf objdump nm check

all: $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Save the configuration as a file, so make understands if it changed.
##------------------------------------------------------------------------------
$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)

##------------------------------------------------------------------------------
## Compile the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF_RAW): $(KERNEL_ELF_RAW_DEPS)
	$(call color_header, "Compiling kernel ELF - $(BSP)")
	@echo "RUSTC_CMD: RUSTFLAGS=\"$(RUSTFLAGS_PEDANTIC)\" $(RUSTC_CMD)"
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

##------------------------------------------------------------------------------
## Precompute the kernel translation tables and patch them into the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF_TTABLES): $(KERNEL_ELF_TTABLES_DEPS)
	$(call color_header, "Precomputing kernel translation tables and patching kernel ELF")
	@cp $(KERNEL_ELF_RAW) $(KERNEL_ELF_TTABLES)
	@$(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $(KERNEL_ELF_TTABLES)

##------------------------------------------------------------------------------
## Generate kernel symbols and patch them into the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF_TTABLES_SYMS): $(KERNEL_ELF_TTABLES_SYMS_DEPS)
	$(call color_header, "Generating kernel symbols and patching kernel ELF")
	@$(MAKE) --no-print-directory -f kernel_symbols.mk

##------------------------------------------------------------------------------
## Generate the stripped kernel binary
##------------------------------------------------------------------------------
$(KERNEL_BIN): $(KERNEL_ELF_TTABLES_SYMS)
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(KERNEL_ELF_TTABLES_SYMS) $(KERNEL_BIN)
	$(call color_progress_prefix, "Name")
	@echo $(KERNEL_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(KERNEL_BIN))

##------------------------------------------------------------------------------
## Generate the documentation
##------------------------------------------------------------------------------
doc: clean
	$(call color_header, "Generating docs")
	@$(DOC_CMD) --document-private-items --open

##------------------------------------------------------------------------------
## Run the kernel in QEMU
##------------------------------------------------------------------------------
ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu:
	$(call color_header, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu: $(KERNEL_BIN)
	$(call color_header, "Launching QEMU")
	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

endif

##------------------------------------------------------------------------------
## Push the kernel to the real HW target
##------------------------------------------------------------------------------
chainboot: $(KERNEL_BIN)
	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD) --features test_build --tests \
                --manifest-path $(KERNEL_MANIFEST)

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
	rm -rf target $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Run readelf
##------------------------------------------------------------------------------
readelf: $(KERNEL_ELF)
	$(call color_header, "Launching readelf")
	@$(DOCKER_TOOLS) $(READELF_BINARY) --headers $(KERNEL_ELF)

##------------------------------------------------------------------------------
## Run objdump
##------------------------------------------------------------------------------
objdump: $(KERNEL_ELF)
	$(call color_header, "Launching objdump")
	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                --section .text   \
                --section .rodata \
                $(KERNEL_ELF) | rustfilt

##------------------------------------------------------------------------------
## Run nm
##------------------------------------------------------------------------------
nm: $(KERNEL_ELF)
	$(call color_header, "Launching nm")
	@$(DOCKER_TOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort | rustfilt



##--------------------------------------------------------------------------------------------------
## Debugging targets
##--------------------------------------------------------------------------------------------------
.PHONY: jtagboot openocd gdb gdb-opt0

##------------------------------------------------------------------------------
## Push the JTAG boot image to the real HW target
##------------------------------------------------------------------------------
jtagboot:
ifeq ($(BSP),rpi5)
	$(call color_header, "Building RPi5 Halt Stub")
	@$(STUB_MAKE_CMD)
	@cp $(JTAG_BOOT_IMAGE) kernel8.img
	$(call color_progress_prefix, "Finished")
	@echo "Created kernel8.img. Please copy this to your SD card."
else
	@$(DOCKER_JTAGBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(JTAG_BOOT_IMAGE)
endif

##------------------------------------------------------------------------------
## Start OpenOCD session
##------------------------------------------------------------------------------
openocd:
	$(call color_header, "Launching OpenOCD")
	@$(DOCKER_OPENOCD) openocd $(OPENOCD_ARG)

##------------------------------------------------------------------------------
## Start GDB session
##------------------------------------------------------------------------------
gdb-opt0: RUSTC_MISC_ARGS += -C opt-level=0
gdb gdb-opt0: $(KERNEL_ELF)
	$(call color_header, "Launching GDB")
ifeq ($(BSP),rpi5)
	@$(GDB_CMD) -q -x $(GDB_INIT_FILE) $(KERNEL_ELF)
else
	@$(GDB_CMD) -q $(KERNEL_ELF)
endif



##--------------------------------------------------------------------------------------------------
## Testing targets
##--------------------------------------------------------------------------------------------------
.PHONY: test test_boot test_unit test_integration

test_unit test_integration: FEATURES += --features test_build

##------------------------------------------------------------------------------
## Run boot test
##------------------------------------------------------------------------------
test_boot: $(KERNEL_BIN)
ifeq ($(BSP),rpi5)
	@$(call color_header, "Skipping boot test for $(BSP) (no QEMU support)")
else ifeq ($(QEMU_MACHINE_TYPE),)
	@$(call color_header, "$(QEMU_MISSING_STRING)")
else
	@$(call color_header, "Boot test - $(BSP)")
	@$(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
endif

##------------------------------------------------------------------------------
## Helpers for unit and integration test targets
##------------------------------------------------------------------------------
define KERNEL_TEST_RUNNER
#!/usr/bin/env bash

    # The cargo test runner seems to change into the crate under test's directory. Therefore, ensure
    # this script executes from the root.
    cd $(shell pwd)

    TEST_ELF=$$(echo $$1 | sed -e 's/.*target/target/g')
    TEST_ELF_SYMS="$${TEST_ELF}_syms"
    TEST_BINARY=$$(echo $$1.img | sed -e 's/.*target/target/g')

    $(DOCKER_TOOLS) $(EXEC_TT_TOOL) $(BSP) $$TEST_ELF > /dev/null

    # This overrides the two ENV variables. The other ENV variables that are required as input for
    # the .mk file are set already because they are exported by this Makefile and this script is
    # started by the same.
    KERNEL_SYMBOLS_INPUT_ELF=$$TEST_ELF           \
        KERNEL_SYMBOLS_OUTPUT_ELF=$$TEST_ELF_SYMS \
        $(MAKE) --no-print-directory -f kernel_symbols.mk > /dev/null 2>&1

    $(OBJCOPY_CMD) $$TEST_ELF_SYMS $$TEST_BINARY
    $(DOCKER_TEST) $(EXEC_TEST_DISPATCH) $(EXEC_QEMU) $(QEMU_TEST_ARGS) -kernel $$TEST_BINARY
endef

export KERNEL_TEST_RUNNER

define test_prepare
    @mkdir -p target
    @echo "$$KERNEL_TEST_RUNNER" > target/kernel_test_runner.sh
    @chmod +x target/kernel_test_runner.sh
endef

##------------------------------------------------------------------------------
## Run unit test(s)
##------------------------------------------------------------------------------
test_unit:
ifeq ($(BSP),rpi5)
	@echo "## Compiling unit test(s) for $(BSP), but not running."
	@echo "Due to no QEMU support, tests are built but not executed."
	@echo "Load and run the test binaries on-target manually for verification."
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD) --no-run --lib
else
	@$(call color_header, "Compiling unit test(s) - $(BSP)")
	@$(call test_prepare)
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD) --lib
endif

##------------------------------------------------------------------------------
## Run integration test(s)
##------------------------------------------------------------------------------
test_integration:
ifeq ($(BSP),rpi5)
	@echo "## Compiling integration test(s) for $(BSP), but not running."
	@echo "Due to no QEMU support, tests are built but not executed."
	@echo "Load and run the test binaries on-target manually for verification."
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD) --no-run $(TEST_ARG)
else
	@$(call color_header, "Compiling integration test(s) - $(BSP)")
	@$(call test_prepare)
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD) $(TEST_ARG)
endif

test: test_boot test_unit test_integration
