use crate::hal;
use crate::drivers::{
    button::Buttons,
    delay::Delay,
    lcd::{Lcd, LcdPins},
    led::Leds,
    serial::SerialPort,
};
use cortex_m::peripheral::Peripherals as CorePeripherals;
use hal::pac;
use hal::prelude::*;
use hal::rcc::Clocks;

/// 聚合板级外设初始化，基于 stm32f4xx-hal。
pub struct Board {
    pub clocks: Clocks,
    pub delay: Delay,
    pub leds: Leds,
    pub buttons: Buttons,
    pub serial: SerialPort,
    pub lcd: Lcd,
}

impl Board {
    pub fn new() -> Self {
        let dp = pac::Peripherals::take().expect("pac already taken");
        let cp = CorePeripherals::take().expect("core already taken");

        let rcc = dp.RCC.constrain();
        // 外部 25MHz 晶振 → 168MHz SYSCLK，对齐参考 C 示例与板卡硬件。
        let cfg = hal::rcc::Config::default()
            .use_hse(25.MHz())
            .sysclk(168.MHz())
            .pclk1(42.MHz())
            .pclk2(84.MHz());
        let mut rcc = rcc.freeze(cfg);
        let clocks = rcc.clocks;

        let mut delay = Delay::new(cp.SYST, &clocks);

        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);
        let gpiof = dp.GPIOF.split(&mut rcc);
        let gpiod = dp.GPIOD.split(&mut rcc);
        let gpioe = dp.GPIOE.split(&mut rcc);
        let gpiog = dp.GPIOG.split(&mut rcc);

        // LEDs: PC0, PF10, PB0, PB1（低电平点亮，初始化时关闭）
        let mut leds = Leds::new(
            gpioc.pc0.into_push_pull_output(),
            gpiof.pf10.into_push_pull_output(),
            gpiob.pb0.into_push_pull_output(),
            gpiob.pb1.into_push_pull_output(),
        );
        leds.all_off();

        // 按键：PE2/PE3/PE4，PA0，使用上拉输入
        let pe2 = gpioe.pe2.into_pull_up_input();
        let pe3 = gpioe.pe3.into_pull_up_input();
        let pe4 = gpioe.pe4.into_pull_up_input();
        let pa0 = gpioa.pa0.into_pull_up_input();
        let buttons = Buttons::new(pe2, pe3, pe4, pa0);

        // LCD pins拆出后传入
        let lcd_pins = LcdPins {
            pd0: gpiod.pd0,
            pd1: gpiod.pd1,
            pd4: gpiod.pd4,
            pd5: gpiod.pd5,
            pd8: gpiod.pd8,
            pd9: gpiod.pd9,
            pd10: gpiod.pd10,
            pd14: gpiod.pd14,
            pd15: gpiod.pd15,
            pe7: gpioe.pe7,
            pe8: gpioe.pe8,
            pe9: gpioe.pe9,
            pe10: gpioe.pe10,
            pe11: gpioe.pe11,
            pe12: gpioe.pe12,
            pe13: gpioe.pe13,
            pe14: gpioe.pe14,
            pe15: gpioe.pe15,
            pg0: gpiog.pg0,
            pg6: gpiog.pg6,
            pg12: gpiog.pg12,
        };

        // 串口：USART1 TX=PA9, RX=PA10，115200 8N1
        let mut tx = gpioa.pa9.into_alternate::<7>();
        tx.set_speed(hal::gpio::Speed::VeryHigh);
        let mut rx = gpioa.pa10.into_alternate::<7>();
        rx.set_speed(hal::gpio::Speed::VeryHigh);
        let serial = SerialPort::new(dp.USART1, tx, rx, &mut rcc, 115_200.bps());

        // LCD：FSMC 16bit 总线 + SSD1963 初始化（480x272）。
        let mut lcd = Lcd::new(dp.FSMC, lcd_pins);
        lcd.init(&mut delay);

        Self {
            clocks,
            delay,
            leds,
            buttons,
            serial,
            lcd,
        }
    }
}
