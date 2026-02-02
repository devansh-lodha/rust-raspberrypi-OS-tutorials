# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2025 Devansh Lodha <devanshlodha12@gmail.com>

# This Makefile is designed to be called from the parent directory's
# Docker environment, which provides the aarch64-elf toolchain.

CROSS_COMPILE ?= aarch64-elf-
OBJCOPY = $(CROSS_COMPILE)objcopy
AS = $(CROSS_COMPILE)as
LD = $(CROSS_COMPILE)ld

IMG_NAME = halt_stub.img
ELF_NAME = halt_stub.elf
OBJ_NAME = start.o

.PHONY: all clean

all: $(IMG_NAME)

$(IMG_NAME): $(ELF_NAME)
	@echo "--- Building Halt Stub Image ---"
	@$(OBJCOPY) -O binary $(ELF_NAME) $(IMG_NAME)

$(ELF_NAME): $(OBJ_NAME)
	@$(LD) -T linker.ld -o $(ELF_NAME) $(OBJ_NAME)

$(OBJ_NAME): start.s
	@$(AS) -o $(OBJ_NAME) start.s

clean:
	@rm -f $(IMG_NAME) $(ELF_NAME) $(OBJ_NAME)