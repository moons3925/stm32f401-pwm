#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use cortex_m::delay;    // (1)Delayを使う
use stm32f4::stm32f401;

#[entry]
fn main() -> ! {

    let dp = stm32f401::Peripherals::take().unwrap();   // (2)デバイス用Peripheralsの取得
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();    // (3)cortex-m Peripheralsの取得
    let mut delay = delay::Delay::new(cp.SYST, 84000000_u32);   // (4)Delayの生成
    clock_init(&dp);    // (5)クロック関連の初期化
    tim2_init(&dp);     // (6)TIM2の初期化
    gpioa5_init(&dp);   // (7)GPIOAの初期化
    tim2_start(&dp);    // (8)PWM スタート
    loop {
        tim2_change_duty(&dp, 500_u32);     // (9)Duty を 50% に変更する
        delay.delay_ms(2000_u32);           // (10)delay 2000msec
        tim2_change_duty(&dp, 100_u32);     // (11)High(LED On)の期間を 10% に変更して暗くする
        delay.delay_ms(1000_u32);           // (12)delay 1000msec
    }
}

fn clock_init(dp: &stm32f401::Peripherals) {

    // PLLSRC = HSI: 16MHz (default)
    dp.RCC.pllcfgr.modify(|_, w| w.pllp().div4());      // (13)P=4
    dp.RCC.pllcfgr.modify(|_, w| unsafe { w.plln().bits(336) });    // (14)N=336
    // PLLM = 16 (default)

    dp.RCC.cfgr.modify(|_, w| w.ppre1().div2());        // (15) APB1 PSC = 1/2
    dp.RCC.cr.modify(|_, w| w.pllon().on());            // (16)PLL On
    while dp.RCC.cr.read().pllrdy().is_not_ready() {    // (17)安定するまで待つ
        // PLLがロックするまで待つ (PLLRDY)
    }

    // データシートのテーブル15より
    dp.FLASH.acr.modify(|_,w| w.latency().bits(2));    // (18)レイテンシの設定: 2ウェイト

    dp.RCC.cfgr.modify(|_,w| w.sw().pll());     // (19)sysclk = PLL
    while !dp.RCC.cfgr.read().sws().is_pll() {  // (20)SWS システムクロックソースがPLLになるまで待つ
    }
}

fn gpioa5_init(dp: &stm32f401::Peripherals) {
    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().enabled());    // (21)GPIOAのクロックを有効にする
    dp.GPIOA.moder.modify(|_, w| w.moder5().alternate());   // (22)GPIOA5をオルタネートに設定    
    dp.GPIOA.afrl.modify(|_, w| w.afrl5().af1());           // (23)GPIOA5をAF1に設定    
}

fn tim2_init(dp: &stm32f401::Peripherals) {
    dp.RCC.apb1enr.modify(|_,w| w.tim2en().enabled());          // (24)TIM2のクロックを有効にする
    dp.TIM2.psc.modify(|_, w| unsafe { w.bits(84 - 1) });       // (25)プリスケーラの設定

    // 周波数はここで決める
    dp.TIM2.arr.modify(|_, w| unsafe { w.bits(1000 - 1) });     // (26)ロードするカウント値
    dp.TIM2.ccmr1_output().modify(|_, w| w.oc1m().pwm_mode1()); // (27)出力比較1 PWMモード1

    // Duty比はここで決まる
    dp.TIM2.ccr1.modify(|_, w| unsafe { w.bits(500 - 1) });     // (28)キャプチャ比較モードレジスタ1
}

fn tim2_start(dp: &stm32f401::Peripherals) {
    dp.TIM2.cr1.modify(|_, w| w.cen().enabled());   // (29)カウンタ有効
    dp.TIM2.ccer.modify(|_, w| w.cc1e().set_bit()); // (30)キャプチャ比較1出力イネーブル
}

// duty : 1 ～ 1000
// 値が 0 ～ 999 の範囲で設定されるように制限しておく

fn tim2_change_duty(dp: &stm32f401::Peripherals, duty: u32) {
    let config;
    if duty == 0 {
        config = 1;
    }
    else if duty > 1000 {
        config = 1000;
    }
    else {
        config = duty;
    }
    dp.TIM2.ccr1.modify(|_, w| unsafe { w.bits(config - 1) });     // (31)キャプチャ比較モードレジスタ1
}
