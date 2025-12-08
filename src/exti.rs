//! 按键外部中断配置（PE2 -> EXTI2, PA0 -> EXTI0），参考实验2。

use crate::hal;
use hal::pac;

/// 配置 EXTI 线 0/2 为下降沿触发，未自动使能 NVIC。
pub fn configure_exti_for_keys(syscfg: &mut pac::SYSCFG, exti: &mut pac::EXTI) {
    // 选择端口
    syscfg.exticr1().modify(|_, w| unsafe {
        w.exti0().bits(0b0000) // PA0
    });
    syscfg.exticr1().modify(|_, w| unsafe {
        w.exti2().bits(0b0100) // PE2
    });

    // 下降沿触发
    exti.ftsr().modify(|_, w| w.tr0().set_bit().tr2().set_bit());
    // 屏蔽中断解除
    exti.imr().modify(|_, w| w.mr0().set_bit().mr2().set_bit());
    // 清 pending
    exti.pr().write(|w| w.pr0().bit(true).pr2().bit(true));
}

/// 清除 EXTI0/EXTI2 的 pending 位。
pub fn clear_key_pending(exti: &mut pac::EXTI) {
    exti.pr().write(|w| w.pr0().bit(true).pr2().bit(true));
}
