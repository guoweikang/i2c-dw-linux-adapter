// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

mod core_base;
pub(crate) use dw_apb_i2c::{I2cDwDriverConfig, I2cDwMasterDriver};
pub(crate) use i2c_common::{msg, I2cSpeedMode, I2cTiming};

use kernel::{
    irq,
    driver,
    bindings, c_str,
    device::{Device, RawDevice},
    i2c::*,
    module_platform_driver, of, platform,
    prelude::*,
    sync::{Arc, ArcBorrow},
};

module_platform_driver! {
      type: DwI2cDriver,
      name: "i2c_designware",
      license: "GPL",
      initcall: "subsys",
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

#[pin_data]
struct DwI2cData {
    #[pin]
    i2c_adapter: I2cAdapter<DwI2cAlgo>,
    #[pin]
    driver: I2cDwMasterDriver,
    irq: irq::Registration::<DwI2cIrqHandler>,
}

impl driver::DeviceRemoval for DwI2cData {
    fn device_remove(&self) {
        pr_info!("unimplement DwI2cData Remove");
    }
}

impl DwI2cData {
    fn new(driver: I2cDwMasterDriver,
        parent: *mut bindings::device,
        of_node: *mut bindings::device_node,
        irq: u32,
    ) -> Arc<Self> {
        Arc::pin_init(pin_init!(&this in Self {
            i2c_adapter <- I2cAdapter::<DwI2cAlgo>::new(
                c_str!("Synopsys DesignWare I2C adapter"),
                &THIS_MODULE,
                parent,
                &I2C_DW_QUIRKS as *const bindings::i2c_adapter_quirks,
                //SAFETY: init by adapter
                unsafe {&mut I2C_DW_ALGO as *mut bindings::i2c_algorithm},
                of_node), 
            driver: driver,
            irq: irq::Registration::<DwI2cIrqHandler>::try_new(irq,
                //SAFETY: this is always valid
                unsafe {Arc::from_raw(this.as_ptr())},
                irq::flags::SHARED|irq::flags::COND_SUSPEND,
                fmt!("dw_i2c_irq_{irq}")).unwrap(),
        })).unwrap()
    }
}

struct DwI2cAlgo;
#[vtable]
impl I2cAlgo for DwI2cAlgo {
    type Data = Arc<DwI2cData>;
    fn master_xfer(data: ArcBorrow<'_, DwI2cData>, msgs: &I2cMsg, msg_num: usize) -> Result<i32> {
        let trans_msgs = msgs.into_array(msg_num, |x: &mut bindings::i2c_msg| {
            msg::I2cMsg::new(x.addr, msg::I2cMsgFlags::from_bits(x.flags).unwrap(), x.buf, x.len as usize)
        })?;

        let master_driver = &data.driver;
        master_driver.master_transfer(trans_msgs)
    }

    fn functionality(data: ArcBorrow<'_, DwI2cData>) -> u32 {
        let master_driver = &data.driver;
        master_driver.get_functionality().bits()
    }
}

struct DwI2cIrqHandler;
impl irq::Handler for DwI2cIrqHandler {
    type Data = Arc<DwI2cData>;

    fn handle_irq(data: ArcBorrow<'_, DwI2cData>) -> irq::Return {
        let master_driver = &data.driver;
        master_driver.irq_handler()
    }
}

struct DwI2cDriver;
impl platform::Driver for DwI2cDriver {
    type Data = Arc<DwI2cData>;

    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(
        pdev: &mut platform::Device,
        _id_info: Option<&Self::IdInfo>,
    ) -> Result<Arc<DwI2cData>> {
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
        let driver_config = I2cDwDriverConfig::new(timing, clk_rate_khz);
        let mut i2c_master_driver = I2cDwMasterDriver::new(driver_config, reg_base);
        i2c_master_driver.setup()?;

        // create data
        let data = DwI2cData::new(i2c_master_driver, 
            dev.raw_device(), 
            pdev.of_node(), 
            irq as u32,
        );
        
        //register adapter
        (&(data.i2c_adapter)).add_numbered_adapter(data.clone())?;
        Ok(data)
    }
}
