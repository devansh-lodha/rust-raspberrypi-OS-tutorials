/*
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2025 Devansh Lodha <devanshlodha12@gmail.com>
 */

ENTRY(_start)
SECTIONS
{
    . = 0x80000;
    .text : { *(.text.boot) }
}