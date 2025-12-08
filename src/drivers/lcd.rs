//! SSD1963 LCD 驱动（480x272，FSMC 16bit 并口），参考实验5 C 代码。

use crate::hal;
use core::convert::Infallible;
use embedded_graphics_core::geometry::Size;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::Pixel;
use hal::gpio::{gpiod, gpioe, gpiog, Input, Output, PushPull};
use hal::pac;

const LCD_BASE: u32 = 0x6C00_0000 | 0x0000_07FE;
const SSD1963_ID: u16 = 0x1963;
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

#[repr(C)]
struct Regs {
    reg: u16,
    ram: u16,
}

/// 8080/FSMC 并口所需的引脚集合。
pub struct LcdPins {
    pub pd0: gpiod::PD0<Input>,
    pub pd1: gpiod::PD1<Input>,
    pub pd4: gpiod::PD4<Input>,
    pub pd5: gpiod::PD5<Input>,
    pub pd8: gpiod::PD8<Input>,
    pub pd9: gpiod::PD9<Input>,
    pub pd10: gpiod::PD10<Input>,
    pub pd14: gpiod::PD14<Input>,
    pub pd15: gpiod::PD15<Input>,
    pub pe7: gpioe::PE7<Input>,
    pub pe8: gpioe::PE8<Input>,
    pub pe9: gpioe::PE9<Input>,
    pub pe10: gpioe::PE10<Input>,
    pub pe11: gpioe::PE11<Input>,
    pub pe12: gpioe::PE12<Input>,
    pub pe13: gpioe::PE13<Input>,
    pub pe14: gpioe::PE14<Input>,
    pub pe15: gpioe::PE15<Input>,
    pub pg0: gpiog::PG0<Input>,
    pub pg6: gpiog::PG6<Input>,
    pub pg12: gpiog::PG12<Input>,
}

pub struct Lcd {
    regs: *mut Regs,
    pub width: u16,
    pub height: u16,
    backlight: gpiog::PG6<Output<PushPull>>,
    _fsmc: pac::FSMC,
}

impl Lcd {
    pub fn new(fsmc: pac::FSMC, pins: LcdPins) -> Self {
        // GPIO 复用为 FSMC AF12
        pins.pd0.into_alternate::<12>();
        pins.pd1.into_alternate::<12>();
        pins.pd4.into_alternate::<12>();
        pins.pd5.into_alternate::<12>();
        pins.pd8.into_alternate::<12>();
        pins.pd9.into_alternate::<12>();
        pins.pd10.into_alternate::<12>();
        pins.pd14.into_alternate::<12>();
        pins.pd15.into_alternate::<12>();

        pins.pe7.into_alternate::<12>();
        pins.pe8.into_alternate::<12>();
        pins.pe9.into_alternate::<12>();
        pins.pe10.into_alternate::<12>();
        pins.pe11.into_alternate::<12>();
        pins.pe12.into_alternate::<12>();
        pins.pe13.into_alternate::<12>();
        pins.pe14.into_alternate::<12>();
        pins.pe15.into_alternate::<12>();

        pins.pg0.into_alternate::<12>(); // A10 -> 8080 RS
        pins.pg12.into_alternate::<12>(); // NE4
        let backlight = pins.pg6.into_push_pull_output(); // LCD_LED

        // 使能 FSMC 时钟
        unsafe {
            let rcc = &*pac::RCC::ptr();
            rcc.ahb3enr().modify(|_, w| w.fsmcen().set_bit());
        }

        // 配置 FSMC Bank1_NORSRAM4，16bit
        let f = unsafe { &*pac::FSMC::ptr() };
        // 先禁用 bank
        f.bcr4().modify(|_, w| w.mbken().clear_bit());

        // 读时序（较宽容）
        f.btr4().modify(|_, w| unsafe {
            w.addset().bits(0xF);
            w.addhld().bits(0);
            w.datast().bits(60);
            w.busturn().bits(0);
            w.clkdiv().bits(0);
            w.datlat().bits(0);
            w.accmod().bits(0)
        });
        // 写时序（较快）
        f.bwtr4().modify(|_, w| unsafe {
            w.addset().bits(9);
            w.addhld().bits(0);
            w.datast().bits(8);
            w.busturn().bits(0);
            w.accmod().bits(0)
        });
        // 基本控制
        f.bcr4().modify(|_, w| unsafe {
            w.muxen().clear_bit(); // 地址数据不复用
            w.mtyp().bits(0b00); // SRAM
            w.mwid().bits(0b01); // 16bit
            w.wren().set_bit(); // 允许写
            w.extmod().set_bit(); // 读写不同定时
            w
        });
        f.bcr4().modify(|_, w| w.mbken().set_bit()); // 使能 bank

        Lcd {
            regs: LCD_BASE as *mut Regs,
            width: WIDTH,
            height: HEIGHT,
            backlight,
            _fsmc: fsmc,
        }
    }

    pub fn init(&mut self, delay: &mut crate::drivers::delay::Delay) {
        // 打开背光
        self.backlight.set_high();
        let id = self.read_id();
        if id != SSD1963_ID {
            // 若读不到 ID，仍尝试初始化
        }

        // PLL 配置
        self.write_reg(0x00E2);
        self.write_data(0x001D);
        self.write_data(0x0002);
        self.write_data(0x0004);

        self.write_reg(0x00E0);
        self.write_data(0x0001);
        delay.ms(1);
        self.write_reg(0x00E0);
        self.write_data(0x0003);
        delay.ms(5);

        self.write_reg(0x0001); // software reset
        delay.ms(5);

        self.write_reg(0x00E6); // 像素时钟
        self.write_data(0x0001);
        self.write_data(0x000A);
        self.write_data(0x003C);

        // 设置分辨率与时序
        self.write_reg(0x00B0);
        self.write_data(0x0020); // 24位接口
        self.write_data(0x0000); // TFT
        self.write_data((WIDTH - 1) >> 8);
        self.write_data((WIDTH - 1) & 0xFF);
        self.write_data((HEIGHT - 1) >> 8);
        self.write_data((HEIGHT - 1) & 0xFF);
        self.write_data(0x0000);

        let ht = WIDTH + 46 + 210;
        self.write_reg(0x00B4); // 水平
        self.write_data((ht - 1) >> 8);
        self.write_data((ht - 1) & 0xFF);
        self.write_data(46 >> 8);
        self.write_data(46 & 0xFF);
        self.write_data(0); // 脉宽
        self.write_data(0x00);
        self.write_data(0x00);
        self.write_data(0x00);

        let vt = HEIGHT + 23 + 22;
        self.write_reg(0x00B6); // 垂直
        self.write_data((vt - 1) >> 8);
        self.write_data((vt - 1) & 0xFF);
        self.write_data(23 >> 8);
        self.write_data(23 & 0xFF);
        self.write_data(22 - 1);
        self.write_data(0x00);
        self.write_data(0x00);

        // 背光 PWM
        self.write_reg(0x00D0);
        self.write_data(0x000D);
        self.write_reg(0x00BE);
        self.write_data(0x0006);
        self.write_data(0x00FE);
        self.write_data(0x0001);
        self.write_data(0x00F0);
        self.write_data(0x0000);
        self.write_data(0x0000);

        self.write_reg(0x00B8);
        self.write_data(0x0003);
        self.write_data(0x0001);
        self.write_reg(0x00BA);
        self.write_data(0x0001);

        self.write_reg(0x0036); // 地址模式
        self.write_data(0x0000);

        self.write_reg(0x00F0); // 16bit(565)
        self.write_data(0x0003);

        self.write_reg(0x0029); // 显示 ON
    }

    pub fn clear(&mut self, color: u16) {
        self.set_window(0, 0, self.width - 1, self.height - 1);
        self.write_reg(0x002C);
        for _ in 0..(self.width as u32 * self.height as u32) {
            self.write_data(color);
        }
    }

    pub fn set_window(&mut self, xs: u16, ys: u16, xe: u16, ye: u16) {
        self.write_reg(0x002A);
        self.write_data(xs >> 8);
        self.write_data(xs & 0xFF);
        self.write_data(xe >> 8);
        self.write_data(xe & 0xFF);

        self.write_reg(0x002B);
        self.write_data(ys >> 8);
        self.write_data(ys & 0xFF);
        self.write_data(ye >> 8);
        self.write_data(ye & 0xFF);
    }

    fn write_reg(&self, reg: u16) {
        let reg_ptr = self.regs as *mut u16;
        unsafe { core::ptr::write_volatile(reg_ptr, reg) }
    }

    fn write_data(&self, data: u16) {
        let data_ptr = (self.regs as *mut u16).wrapping_add(1);
        unsafe { core::ptr::write_volatile(data_ptr, data) }
    }

    fn read_data(&self) -> u16 {
        let data_ptr = (self.regs as *mut u16).wrapping_add(1);
        unsafe { core::ptr::read_volatile(data_ptr) }
    }

    fn write_reg_data(&self, reg: u16, data: u16) {
        self.write_reg(reg);
        self.write_data(data);
    }

    fn read_id(&self) -> u16 {
        self.write_reg(0x0000);
        let _ = self.read_data();
        let id_high = self.read_data();
        let id_low = self.read_data();
        (id_high << 8) | (id_low & 0xFF)
    }
}

impl OriginDimensions for Lcd {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl DrawTarget for Lcd {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x < 0 || point.y < 0 {
                continue;
            }
            let (x, y) = (point.x as u16, point.y as u16);
            if x >= self.width || y >= self.height {
                continue;
            }
            self.set_window(x, y, x, y);
            self.write_reg(0x002C);
            self.write_data(color.into_storage());
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear(color.into_storage());
        Ok(())
    }
}
