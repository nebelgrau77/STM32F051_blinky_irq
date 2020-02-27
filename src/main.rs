// blinking built-in LED on an STM32F051C8T6 board
// with a timer and interrupts

#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    gpio::*,
    prelude::*,
    stm32::{interrupt, Interrupt, Peripherals, TIM3},
    time::Hertz,
    timers::*,
};

use cortex_m_rt::entry;

use core::cell::RefCell;
use cortex_m::{interrupt::Mutex, peripheral::Peripherals as c_m_Peripherals};

// A type definition for the GPIO pin to be used for our LED
// Using built-in LED on PC13
type LEDPIN = gpioc::PC13<Output<PushPull>>;

// Make LED pins globally available
static GLED: Mutex<RefCell<Option<LEDPIN>>> = Mutex::new(RefCell::new(None));

// Make timer interrupts registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM3>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM3() {
    static mut LED: Option<LEDPIN> = None;
    static mut INT: Option<Timer<TIM3>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            GLED.borrow(cs).replace(None).unwrap()
        })
    });

    let int = INT.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            GINT.borrow(cs).replace(None).unwrap()
        })
    });

    led.toggle().ok();
    int.wait().ok();
}


#[entry]
fn main() -> ! {

    let mut p = Peripherals::take().unwrap();
    let mut cp = c_m_Peripherals::take().unwrap();

    cortex_m::interrupt::free(move |cs| 
        {
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
            
            let gpioc = p.GPIOC.split(&mut rcc);

            // (Re-)configure PC13 as output
            let led = gpioc.pc13.into_push_pull_output(cs);

            // Move the pin into our global storage
            *GLED.borrow(cs).borrow_mut() = Some(led);

            // Set up a timer expiring after 200ms, that is 5 times per second
            let mut timer = Timer::tim3(p.TIM3, Hertz(5), &mut rcc);
            
            // Generate an interrupt when the timer expires
            timer.listen(Event::TimeOut);

            // Move the timer into our global storage
            *GINT.borrow(cs).borrow_mut() = Some(timer);

            // Enable TIM3 IRQ, set priority 1 and clear any pending IRQs
            
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM3, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
               
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

        });

        loop
            {continue;}

}   