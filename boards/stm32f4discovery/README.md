stm32f4discovery: Platform-Specific Instructions
=====================================

(http://www.st.com/en/evaluation-tools/stm32f4discovery.html)


## Flashing the kernel

To program the Tock kernel onto the Discovery, `cd` into the `boards/stm32f4discovery` directory
and run:

```bash
make flash-debug
```

This will build `boards/stm32f4discovery/target/thumbv7em-none-eabi/debug/stm32f4discovery` and use `openocd` to
flash it to the board.

## Apps

All user-level code lives in the `userland` subdirectory. This includes a
specially compiled version of newlib, a user-level library for talking to the
kernel and specific drivers and a variety of example applications.

To compile an app, `cd` to the desired app and `make`. For example:

```bash
cd tock/userland/examples/c_hello/
make TOCK_BOARD=stm32f4discovery
```

### Flashing one app

```bash
cd tock/userland/examples/c_hello/
make TOCK_BOARD=stm32f4discovery flash
```

### Flashing multiple apps

```bash
cd userland

APPS="buttons c_hello"
BINS=()

for APP in $APPS; do
  pushd .
  cd "../../tock/userland/examples/${APP}/"
  make TOCK_BOARD=stm32f4discovery
  popd
  BINS+=("../../tock/userland/examples/${APP}/build/cortex-m4/cortex-m4.bin")
done

APP_BASE_ADDR=0x08080000 ./tools/flash/stm32f4.sh "${BINS[@]}"
```

## Debugging using Visual Studio Code

*TODO* (Status: working very well)

Edit `discovery.code-workspace` and point `rust.sysroot` to correct full path
to `.xargo` in this directory.

## Console I/O

The the kernel debugging console and the stdout console is implemented using
[semihosting](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0471g/Bgbjjgij.html).

Huge benefit of semihosting console is that it in contrast to UART console requires no hardware
initialization.

`std_out_init()` initializes the semihosting only if a debugger is connected
so the kernel can run without debugger out-of-box without needing to replace
or remove the semihosting console in code.

To enable semihosting with one needs to add `arm semihosting enable` to `openocd`
initialization commnds e.g.:

```bash
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg -c "init; reset halt; arm semihosting enable"
```

The command can also be passed from `gdb` using `monitor arm semihosting enable` command.
