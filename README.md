# i2c-dw-linux-adapter
Under the cross-kernel driver framework, the Linux driver adaptation layer implemented for designware apb I2C

# How to use

## Clone repo
In your linux directory, clone this project

```shell
cd drivers/i2c/busses
git clone https://github.com/guoweikang/i2c-dw-linux-adapter.git
```

## Linux support Cargo 
The cross-kernel driver framework follows a componentized design and uses cargo to resolve component dependencies,
so it is necessary to add R4L support for cargo construction.

### step1: Patch linux support cargo build

patch on `patches/0001_linux_support_cargo.diff`

### step2: Add Makefile for adapter dir

Add this line into linux/drivers/i2c/busses/Makefile
``` shell
obj-$(CONFIG_RUST)     += i2c-dw-linux-adapter/
```

you can also replace `CONFIG_RUST` with your own defin, like this from original `I2C_DESIGNWARE_PLATFORM` Kconfig

```shell
config I2C_DESIGNWARE_PLATFORM_RUST
	tristate "Synopsys DesignWare Platform with RUST"
    depends on (ACPI && COMMON_CLK) || !ACPI
    depends on RUST
	help
      If you say yes to this option, support will be included for the
      Synopsys DesignWare I2C adapter.

      This driver can also be built as a module.  If so, the module
      will be called i2c-designware-platform.      
```

**note**: if you want to use RUST driver,remeber disable C driver


### step3: add Makefile for adapter that use cargo build 


