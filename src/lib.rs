// SPDX-License-Identifier: GPL-2.0


//! Rust dw_apb_i2c

#![no_std]

use kernel::{module_platform_driver, of, platform, prelude::*};

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
    
    fn probe(_dev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result{
        Ok(())
    }
}
