// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

mod core_base;

pub(crate) use dw_apb_i2c::{timing::I2cTiming, I2cSpeedMode};

use kernel::device::RawDevice;
use kernel::{module_platform_driver, of, platform, prelude::*};

use kernel::device::Device;
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

struct DwI2cDriver;

impl platform::Driver for DwI2cDriver {
    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result {
        let irq = pdev.irq_resource(0)?;
        pdev.pr_info(format_args!("========================== get irq {}", irq));

        let reg_base = pdev.ioremap_resource(0)?;
        pdev.pr_info(format_args!(
            "========================== ioremap ptr {:p}",
            reg_base
        ));

        let dev = Device::from_dev(pdev);
        let timing = core_base::i2c_parse_fw_timings(&dev, false);
        dev.pr_info(format_args!(
            "========================== timing {:?}",
            timing
        ));
        //let bus_freq = device::

        //let i2c_config  = I2cDesignwareDriverConfig::new();
        Ok(())
    }
}
