// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

mod core_base;

pub(crate) use dw_apb_i2c::{I2cTiming, I2cSpeedMode};
pub(crate) use dw_apb_i2c::{I2cDwMasterDriver, I2cDwDriverConfig};

use kernel::{module_platform_driver, of, platform, prelude::*};
use kernel::device::Device;
use kernel::i2c::*;

//use dw_apb_i2c::{I2cDesignwareDriverConfig};

module_platform_driver! {
      type: DwI2cDriver,
      name: "i2c_designware",
      license: "GPL",
}

// Linux Raw id table
kernel::module_of_id_table!(DW_I2C_MOD_TABLE, DW_I2C_OF_MATCH_TABLE);
// R4L IdArray table
kernel::define_of_id_table! {DW_I2C_OF_MATCH_TABLE, (), [
    (of::DeviceId::Compatible(b"snps,designware-i2c"),None),
]}

/*
struct DwI2cDriver {
    pure_driver: I2cDesignwareDriver,
    device: Device
}
*/
struct DwI2cDriver;

impl platform::Driver for DwI2cDriver {
    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result {
        let irq = pdev.irq_resource(0)?;
        let reg_base = pdev.ioremap_resource(0)?;
        let dev = Device::from_dev(pdev);
        let timing = core_base::i2c_parse_fw_timings(&dev, false);

        if i2c_detect_slave_mode(&dev) {
            pr_err!("unimplement dw slave driver");
            return Ok(());
        } 
        
        let driver_config = I2cDwDriverConfig::new(irq, timing);
        let mut i2c_master_driver = I2cDwMasterDriver::new(driver_config, reg_base);
        i2c_master_driver.config_init()?;
        i2c_master_driver.setup()?;
        //let bus_freq = device::
        //let i2c_config  = I2cDesignwareDriverConfig::new();
        Ok(())
    }
}
