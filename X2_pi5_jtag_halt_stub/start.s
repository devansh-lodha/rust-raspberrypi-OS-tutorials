/*
 * SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2025 Devansh Lodha <devanshlodha12@gmail.com>
 */

.section ".text.boot"
.global _start
_start:
    wfe // Wait for event (low-power idle)
    b _start