# PicOS

A real-time operating system from scratch for Raspberry Pi Pico based projects (could easily be adapted for RP2040 support)

Currently implemented is a simple Round Robin based scheduler for tasks within the system. Scheduling happens based on the SysTick interrupt at a configurable rate.

USB HID Keyboard reports have been added as well for basic keyboard emulation.