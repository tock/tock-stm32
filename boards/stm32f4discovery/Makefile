# Makefile for building the tock kernel for the Imix platform

TARGET = thumbv7em-none-eabi
PLATFORM = stm32f4discovery

include ../../tock/boards/Makefile.common

export XARGO_HOME := $(PWD)/.xargo

OPENOCD = openocd -f connect.cfg

.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).bin
	echo "Not implemented"

# upload kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM)
	$(OPENOCD) -c "program $< reset exit"

.PHONY: flash-debug
flash-debug: target/$(TARGET)/debug/$(PLATFORM)
	$(OPENOCD) -c "program $< reset exit"

.PHONY: distclean
distclean: clean
	$(Q)rm -rf .xargo

.PHONY: flash-debug-run
flash-debug-run: flash-debug
	$(OPENOCD) -c "init; reset halt"

.PHONY: run
run:
	$(OPENOCD) -c "init; reset halt"
