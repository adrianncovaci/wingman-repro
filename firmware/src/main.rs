// #![deny(unsafe_code)]
#![no_std]
#![no_main]
// #![feature(type_alias_impl_trait)]

#[macro_use]
mod macros;

mod data;
mod exception_handlers;
mod io;
mod lan8720a;
mod net;
mod oled_display;

use rtic::app;

#[app(device = stm32h7xx_hal::stm32, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use crate::net::ethernet::Ethernet;
    use crate::net::net_storage::NetStorage;
    use crate::oled_display::OledDisplay;
    #[cfg(feature = "defmt")]
    use defmt_rtt as _;
    use firmware_logic::data::user_commands::UserCommands;
    use firmware_logic::{Configuration, ControllerLogic, FirmwareLogic, FirmwareReporting, Wingman2HardwareStatus, Wingman2IOCardStatus, Wingman3HardwareStatus};
    use panic_probe as _;
    use rtic_monotonics::systick::Systick;
    use rtic_monotonics::Monotonic;
    use stm32h7xx_hal::delay::DelayFromCountDownTimer;
    use stm32h7xx_hal::gpio::Speed;
    use stm32h7xx_hal::prelude::*;

    // Set up PLL to 168MHz from 16MHz HSI
    #[shared]
    struct SharedResources {
        hardware_status: Wingman2HardwareStatus,
        card_status: Wingman2IOCardStatus,
        logic: FirmwareLogic,
        reporting: FirmwareReporting,
        user_commands: UserCommands,
        ethernet: Ethernet,
        shared_hardware_status: Wingman3HardwareStatus,
    }

    #[local]
    struct LocalResources {
        oled_display: OledDisplay,
    }

    #[init(local = [card_status: Wingman2IOCardStatus = Wingman2IOCardStatus::default(), net_storage: NetStorage = NetStorage::new() ])]
    fn init(mut cx: init::Context) -> (SharedResources, LocalResources) {
        #[cfg(feature = "rtt")]
        rtt_target::rtt_init_print!();
        rtt_debug!("\n====================================================\nBooting ...");
        let pwr = cx.device.PWR.constrain();
        let power_configuration = pwr.smps().freeze();

        // Link the SRAM3 power state to CPU1
        cx.device.RCC.ahb2enr.modify(|_, w| w.sram3en().set_bit());

        // Initialize system...
        cx.core.SCB.enable_icache();
        cx.core.DWT.enable_cycle_counter();

        // RCC
        let rcc = cx.device.RCC.constrain();
        let mut ccdr = rcc
            .sys_ck(200.MHz())
            .hclk(200.MHz())
            .pclk1(100.MHz())
            .pclk2(100.MHz())
            .pclk4(100.MHz())
            .pll1_q_ck(48.MHz())
            .pll2_p_ck(24.MHz())
            .pll2_q_ck(24.MHz())
            .freeze(power_configuration, &cx.device.SYSCFG);

        let c_ck_mhz = ccdr.clocks.c_ck().to_MHz();
        let syst_calib = 1_000;

        rtt_debug!("Core Initialized");

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 200_000_000, systick_token);

        rtt_debug!("Init complete.");

        // Delay Timer

        let timer14 = cx
            .device
            .TIM14
            .timer(1.kHz(), ccdr.peripheral.TIM14, &ccdr.clocks);
        let mut delay1 = DelayFromCountDownTimer::new(timer14);

        let timer15 = cx
            .device
            .TIM15
            .timer(1.kHz(), ccdr.peripheral.TIM15, &ccdr.clocks);
        let mut delay2 = DelayFromCountDownTimer::new(timer15);

        let timer16 = cx
            .device
            .TIM16
            .timer(1.kHz(), ccdr.peripheral.TIM16, &ccdr.clocks);
        let mut delay3 = DelayFromCountDownTimer::new(timer16);

        // GPIO Banks
        let gpioa = cx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = cx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = cx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = cx.device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = cx.device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = cx.device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = cx.device.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = cx.device.GPIOH.split(ccdr.peripheral.GPIOH);
        let gpioi = cx.device.GPIOI.split(ccdr.peripheral.GPIOI);
        let gpioj = cx.device.GPIOJ.split(ccdr.peripheral.GPIOJ);
        let gpiok = cx.device.GPIOK.split(ccdr.peripheral.GPIOK);
        rtt_debug!("Initializing the display ...");
        let oled_display = OledDisplay::new(
            cx.device.SPI1,
            ccdr.peripheral.SPI1,
            gpiog.pg11,
            gpiod.pd7,
            gpiog.pg9,
            gpiog.pg10,
            gpioj.pj15,
            &mut delay1,
            ccdr.clocks,
        );

        let rmii_ref_clk = gpioa.pa1.into_alternate().speed(Speed::VeryHigh);
        let rmii_mdio = gpioa.pa2.into_alternate().speed(Speed::VeryHigh);
        let rmii_mdc = gpioc.pc1.into_alternate().speed(Speed::VeryHigh);
        let rmii_crs_dv = gpioa.pa7.into_alternate().speed(Speed::VeryHigh);
        let rmii_rxd0 = gpioc.pc4.into_alternate().speed(Speed::VeryHigh);
        let rmii_rxd1 = gpioc.pc5.into_alternate().speed(Speed::VeryHigh);
        let rmii_tx_en = gpiob.pb11.into_alternate().speed(Speed::VeryHigh);
        let rmii_txd0 = gpiob.pb12.into_alternate().speed(Speed::VeryHigh);
        let rmii_txd1 = gpiob.pb13.into_alternate().speed(Speed::VeryHigh);

        let mut rmii_rst = gpioc.pc2.into_push_pull_output();
        rmii_rst.set_low();
        delay1.delay_ms(1u16);
        rmii_rst.set_high();
        delay1.delay_ms(1u16);

        let ethernet_timer = cx
            .device
            .TIM1
            .timer(1.kHz(), ccdr.peripheral.TIM1, &ccdr.clocks);
        let ethernet = Ethernet::new(
            cx.device.ETHERNET_MAC,
            cx.device.ETHERNET_MTL,
            cx.device.ETHERNET_DMA,
            (
                rmii_ref_clk,
                rmii_mdio,
                rmii_mdc,
                rmii_crs_dv,
                rmii_rxd0,
                rmii_rxd1,
                rmii_tx_en,
                rmii_txd0,
                rmii_txd1,
            ),
            ccdr.peripheral.ETH1MAC,
            ethernet_timer,
            &ccdr.clocks,
            cx.local.net_storage,
        );

        ethernet_sync_control_server::spawn().unwrap();
        apply_logic::spawn().unwrap();
        (
            SharedResources {
                hardware_status: Wingman2HardwareStatus::new(Configuration::Unconfigured),
                user_commands: UserCommands::default(),
                logic: FirmwareLogic::default(),
                reporting: FirmwareReporting::default(),
                card_status: Wingman2IOCardStatus::default(),
                ethernet,
                shared_hardware_status: Wingman3HardwareStatus::new(Configuration::Unconfigured),

            },
            LocalResources { oled_display, /*dio*/ },
        )
    }

    #[idle]
    #[inline(never)]
    fn idle(_cx: idle::Context) -> ! {
        rtt_debug!("idle");

        loop {
            cortex_m::asm::nop();
            display_task::spawn().unwrap();
        }
    }

    #[task(priority = 1, local = [oled_display], shared = [ethernet, hardware_status])]
    async fn display_task(mut cx: display_task::Context) {
        // (&mut cx.shared.hardware_status, &mut cx.shared.shared_hardware_status).lock(|hardware_status, shared_hardware_status| {
        //     *shared_hardware_status = hardware_status.clone();
        // });

        cx.shared
            .hardware_status
            .lock(|hardware_status|
        cx.local.oled_display.set_timestamp(hardware_status.now));
        cx.local.oled_display.update();
    }

    #[task(priority = 2, shared = [ethernet, user_commands, hardware_status, reporting])]
    async fn ethernet_sync_control_server(mut cx: ethernet_sync_control_server::Context) {
        loop {
            // let reporting = cx.shared.reporting.lock(|reporting| reporting.clone());
            (&mut cx.shared.ethernet, &mut cx.shared.user_commands, &mut cx.shared.hardware_status, &mut cx.shared.reporting).lock(
                |ethernet, user_commands, hardware_status, reporting| {
                    ethernet.synchronize_control_server_socket(
                        user_commands,
                        hardware_status,
                        reporting,
                    );
                },
            );
            Systick::delay(25.millis().into()).await;
        }
    }

    fn update_hardware_status(hardware_status: &mut Wingman2HardwareStatus) {
        hardware_status.step += 1;
        hardware_status.now = <Systick as Monotonic>::now().into();
    }

    fn apply_status_to_update_hardware(hardware_status: &Wingman2HardwareStatus) {
    }

    #[task(priority = 3, shared = [ethernet, hardware_status, user_commands, logic, reporting])]
    async fn apply_logic(mut cx: apply_logic::Context) {
        (&mut cx.shared.hardware_status, &mut cx.shared.logic).lock(|hardware_status, logic| {
            update_hardware_status(hardware_status);
            logic.initialize(hardware_status);
        });

        loop {
            (&mut cx.shared.hardware_status, &mut cx.shared.logic).lock(
                |hardware_status, logic| {
                    update_hardware_status(hardware_status);
                    cx.shared.user_commands.lock(|user_commands| {
                        logic.apply_user_commands(user_commands, hardware_status);
                    });
                    logic.update(hardware_status);
                    apply_status_to_update_hardware(hardware_status);
                },
            );

            Systick::delay(5.millis().into()).await;
        }
    }
}
