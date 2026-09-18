use esp_hal::gpio::AnyPin;
use esp_hal::peripherals::Peripherals;

pub struct Pins<'a> {
    pub mclk: AnyPin<'a>,
    pub pclk: AnyPin<'a>,
    pub vsync: AnyPin<'a>,
    pub hsync: AnyPin<'a>,

    pub d0: AnyPin<'a>,
    pub d1: AnyPin<'a>,
    pub d2: AnyPin<'a>,
    pub d3: AnyPin<'a>,
    pub d4: AnyPin<'a>,
    pub d5: AnyPin<'a>,
    pub d6: AnyPin<'a>,
    pub d7: AnyPin<'a>,

    pub sda: AnyPin<'a>,
    pub scl: AnyPin<'a>,
}

pub fn s3eye(p: Peripherals) -> Pins<'static> {
    Pins {
        mclk: AnyPin::from(p.GPIO15),
        pclk: AnyPin::from(p.GPIO13),
        vsync: AnyPin::from(p.GPIO6),
        hsync: AnyPin::from(p.GPIO7),

        d0: AnyPin::from(p.GPIO11),
        d1: AnyPin::from(p.GPIO9),
        d2: AnyPin::from(p.GPIO8),
        d3: AnyPin::from(p.GPIO10),
        d4: AnyPin::from(p.GPIO12),
        d5: AnyPin::from(p.GPIO18),
        d6: AnyPin::from(p.GPIO17),
        d7: AnyPin::from(p.GPIO16),

        sda: AnyPin::from(p.GPIO4),
        scl: AnyPin::from(p.GPIO5),
    }
}
