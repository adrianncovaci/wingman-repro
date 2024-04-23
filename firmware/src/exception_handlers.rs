use cortex_m_rt::{exception, ExceptionFrame};

#[allow(non_snake_case, unused)]
unsafe fn DefaultHandler(interrupt_number: i16) -> ! {
    if interrupt_number < 0 {
        rtt_debug!(
            "Core exception with interrupt #{} occurred.",
            interrupt_number
        );
    } else {
        rtt_debug!(
            "Interrupt #{} occurred and doesn't have a specific handler defined.",
            interrupt_number
        );
    }

    loop {
        cortex_m::asm::nop();
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
