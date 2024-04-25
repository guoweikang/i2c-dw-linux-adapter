use crate::{I2cSpeedMode, I2cTiming};
use dw_apb_i2c::I2cTimingBuilder;

use kernel::device::Device;
use kernel::{c_str, str::CStr};

fn i2c_parse_timing<'a, F: FnOnce(u32) -> &'a mut I2cTimingBuilder>(
    dev: &Device,
    propname: &'static CStr,
    f: F,
) {
    let mut val = 0u32;

    // SAFETY: val always valid
    if Ok(()) == unsafe { dev.device_property_read_u32(propname, &mut val) } {
        f(val);
    }
}

/// Create i2c timing config from Device
pub(crate) fn i2c_parse_fw_timings(dev: &Device, use_default: bool) -> I2cTiming {
    let mut builder = I2cTiming::new_builder(I2cSpeedMode::StandMode, use_default);
    i2c_parse_timing(dev, c_str!("clock-frequency"), |x| builder.bus_freq_hz(x));
    i2c_parse_timing(dev, c_str!("i2c-scl-rising-time-ns"), |x| {
        builder.scl_rise_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-scl-falling-time-ns"), |x| {
        builder.scl_fall_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-scl-internal-delay-ns"), |x| {
        builder.scl_int_delay_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-sda-falling-time-ns"), |x| {
        builder.sda_fall_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-sda-hold-time-ns"), |x| {
        builder.sda_hold_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-digital-filter-width-ns"), |x| {
        builder.digital_filter_width_ns(x)
    });
    i2c_parse_timing(dev, c_str!("i2c-analog-filter-cutoff-frequency"), |x| {
        builder.analog_filter_cutoff_freq_hz(x)
    });
    builder.build().unwrap()
}
