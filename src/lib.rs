// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

mod core_base;

pub(crate) use dw_apb_i2c::{I2cDwDriverConfig, I2cDwMasterDriver};
pub(crate) use i2c_common::{msg, I2cSpeedMode, I2cTiming};

use kernel::{
    bindings, c_str, device,
    device::Device,
    i2c,
    i2c::*,
    module_platform_driver, of, platform,
    prelude::*,
    sync::{Arc, ArcBorrow},
};

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

static mut I2C_DW_ALGO: bindings::i2c_algorithm = bindings::i2c_algorithm {
    master_xfer: None,
    master_xfer_atomic: None,
    smbus_xfer: None,
    smbus_xfer_atomic: None,
    functionality: None,
    reg_slave: None,
    unreg_slave: None,
};

static I2C_DW_QUIRKS: bindings::i2c_adapter_quirks = bindings::i2c_adapter_quirks {
    flags: I2C_AQ_NO_ZERO_LEN,
    max_num_msgs: 0,
    max_write_len: 0,
    max_read_len: 0,
    max_comb_1st_msg_len: 0,
    max_comb_2nd_msg_len: 0,
};

struct DwI2cDriver;
type DeviceData = device::Data<i2c::Registration<DwI2cDriver>, I2cDwMasterDriver, ()>;

#[vtable]
impl i2c::I2cAlgo for DwI2cDriver {
    type Data = Arc<DeviceData>;
    fn master_xfer(_data: ArcBorrow<'_, DeviceData>, msgs: &I2cMsg, msg_num: usize) -> Result {
        use core::slice;
        pr_info!("=============== enter master_xfer");
        let trans_msgs = msgs.into_array(msg_num, |x: &mut bindings::i2c_msg| {
            msg::I2cMsg::new(
                x.addr,
                msg::I2cMsgFlags::from_bits(x.flags).unwrap(),
                unsafe { slice::from_raw_parts(x.buf, x.len as usize) },
            )
        })?;

        for m in trans_msgs {
            pr_info!("================ send msg : {:?}", m);
        }

        Ok(())
    }

    fn functionality(data: ArcBorrow<'_, DeviceData>) -> u32 {
        let master_driver = data.resources().unwrap();
        pr_info!(
            "======= get functionality {:?}",
            master_driver.get_functionality().bits()
        );
        master_driver.get_functionality().bits()
    }
}

impl platform::Driver for DwI2cDriver {
    type Data = Arc<DeviceData>;

    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(
        pdev: &mut platform::Device,
        _id_info: Option<&Self::IdInfo>,
    ) -> Result<Arc<DeviceData>> {
        let irq = pdev.irq_resource(0)?;
        let reg_base = pdev.ioremap_resource(0)?;
        let dev = Device::from_dev(pdev);
        let timing = core_base::i2c_parse_fw_timings(&dev, false);

        if i2c_detect_slave_mode(&dev) {
            pr_err!("unimplement dw slave driver");
            return Err(ENODEV);
        }

        // clk
        let clk = dev.devm_clk_get_default_optional()?;
        clk.prepare_enable()?;
        let clk_rate_khz = (clk.get_rate() / 1000) as u32;

        // create master driver instance
        let driver_config = I2cDwDriverConfig::new(irq, timing, clk_rate_khz);
        let mut i2c_master_driver = I2cDwMasterDriver::new(driver_config, reg_base);
        i2c_master_driver.setup()?;

        let data = kernel::new_device_data!(
            i2c::Registration::<DwI2cDriver>::new(),
            i2c_master_driver,
            (),
            "i2c_dw::Registrations"
        )?;

        // SAFETY: General part of the data is pinned when `data` is.
        // From Pin(&mut <UniqueArc<Data>) get  Pin(&mut <Data>)
        let data = Arc::<DeviceData>::from(data);

        i2c::Registration::register(
            data.registrations().ok_or(ENXIO)?.as_pinned_mut(),
            c_str!("Synopsys DesignWare I2C adapter"),
            unsafe { &mut I2C_DW_ALGO },
            &I2C_DW_QUIRKS,
            &dev,
            pdev.of_node(),
            data.clone(),
            &THIS_MODULE,
        )?;

        Ok(data)
    }
}
