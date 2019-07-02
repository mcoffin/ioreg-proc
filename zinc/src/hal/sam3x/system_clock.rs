use ::wait_for;

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum RCFreq {
    MHz_4,
    MHz_8,
    MHz_12,
}

pub enum ClockSource {
    InternalSlow,
    InternalRc(RCFreq),
    Main(Option<u32>),
}

const SLOW_CLOCK_FREQ: u32 = 32_768;

impl ClockSource {
    fn init(&self) {
        use self::ClockSource::*;
        let css = match self {
            &InternalSlow => reg::pmc::mckr::Css::SlowClk,
            &InternalRc(freq) => {
                init_rc_oscillator(freq);
                reg::pmc::mckr::Css::MainClk
            },
            &Main(_) => {
                // FIXME(mcoffin): wait time hard coded
                init_main_oscillator(0x8);
                reg::pmc::mckr::Css::MainClk
            },
        };
        let pmc = unsafe { &reg::PMC };
        pmc.mckr.update().set_css(css);
        wait_for!(pmc.st.get().mckrdy());
    }

    fn freq(&self) -> u32 {
        use self::ClockSource::*;
        use self::RCFreq::*;

        match self {
            &InternalSlow => SLOW_CLOCK_FREQ,
            &InternalRc(MHz_4) => 4_000_000,
            &InternalRc(MHz_8) => 8_000_000,
            &InternalRc(MHz_12) => 12_000_000,
            &Main(Some(f)) => f,
            &Main(None) => mck_freq(),
        }
    }
}

const MAINF_SCALE: u32 = 32_768 / 16;
/// Reads the main clock frequency from the PMC register.
///
/// NOTE: Will wait until the main frequency is measured
pub fn mck_freq() -> u32 {
    let pmc = unsafe { &reg::PMC };

    wait_for!(pmc.mcfr.get().mainfrdy());
    let cycles = pmc.mcfr.get().mainf();
    cycles * MAINF_SCALE
}

pub struct Pll {
    pub mul: u32,
    pub div: u32,
    pub count: u32,
}

impl Pll {
    fn init(&self, src_freq: u32) {
        use self::reg::pmc::mckr::Pres::*;
        let pmc = unsafe { &reg::PMC };

        pmc.pllar.update()
            .set_one(true)
            .set_mula(self.mul as u16)
            .set_diva(self.div as u8)
            .set_pllacount(self.count as u8);
        wait_for!(pmc.st.get().locka());

        /*
        pmc.mckr.set_pres(match self.apply_freq(src_freq) {
            0 ... 84_000_000 => CLK,
            84_000_001 ... 168_000_000 => CLK_2,
            168_000_001 ... 336_000_000 => CLK_4,
            336_000_001 ... 672_000_000 => CLK_8,
            672_000_001 ... 1_344_000_000 => CLK_16,
            _ => panic!("Clock speed too fast!"),
        });
        */
        pmc.mckr.update()
            .set_pres(CLK);
        wait_for!(pmc.st.get().mckrdy());

        pmc.mckr.update()
            .set_css(reg::pmc::mckr::Css::PplaClk);
        wait_for!(pmc.st.get().mckrdy());
    }

    fn apply_freq(&self, freq: u32) -> u32 {
        (freq / self.div) * (self.mul + 1)
    }
}

static FLASH_MAX_FREQ: u32 = 20_000_000;

fn init_flash(clk_freq: u32) {
    let (eefc0, eefc1) = unsafe {
        (&reg::EEFC0, &reg::EEFC1)
    };

    let cycles: u32 = clk_freq / FLASH_MAX_FREQ;
    eefc0.fmr.update().set_fws(cycles as u16);
    eefc1.fmr.update().set_fws(cycles as u16);

    wait_for!(eefc0.fsr.get().fready() &&
              eefc1.fsr.get().fready());
}

/// Initializes the system master clock to be a given clock source optionally
/// with PLL scaling
pub fn init_clock(source: ClockSource, pll: Option<Pll>) -> u32 {
    source.init();

    let src_freq = source.freq();
    let freq = match pll {
        Some(ref p) => p.apply_freq(src_freq),
        _ => src_freq,
    };

    // Init flash
    init_flash(freq);

    match pll {
        Some(p) => p.init(src_freq),
        _ => {},
    }

    freq
}

/// Initializes the system exactly how CMSIS does (84MHz clock)
pub fn temp_boot() {
    use self::reg::pmc::mor::Moscsel::*;
    use self::reg::pmc::mckr::Css::*;
    use self::reg::pmc::mckr::Pres::*;

    // Initialize flash
    let (eefc0, eefc1) = unsafe {
        (&reg::EEFC0, &reg::EEFC1)
    };
    eefc0.fmr.update().set_fws(4);
    eefc1.fmr.update().set_fws(4);

    // Enable main oscillator
    let pmc = unsafe { &reg::PMC };
    match pmc.mor.get().moscsel() {
        MOSCXT => {},
        _ => {
            pmc.mor.update()
                .set_key(MOR_KEY)
                .set_moscxtst(0x8)
                .set_moscrcen(true)
                .set_moscxten(true);
            wait_for!(pmc.st.get().moscxts());
        },
    }

    // Switch to Xtal oscillator
    pmc.mor.update()
        .set_key(MOR_KEY)
        .set_moscxtst(0x8)
        .set_moscrcen(true)
        .set_moscxten(true)
        .set_moscsel(MOSCXT);
    wait_for!(pmc.st.get().moscsels());
    pmc.mckr.update()
        .set_css(MainClk);
    wait_for!(pmc.st.get().mckrdy());

    // Initialize PLLA
    pmc.pllar.update()
        .set_one(true)
        .set_mula(0x3)
        .set_pllacount(0x3f)
        .set_diva(0x1);
    wait_for!(pmc.st.get().locka());

    // Switch to main clock
    pmc.mckr.update()
        .set_pres(CLK_2);
    wait_for!(pmc.st.get().mckrdy());

    // Switch to PLLA
    pmc.mckr.update()
        .set_css(PplaClk);
    wait_for!(pmc.st.get().mckrdy());

    ::hal::cortex_m3::systick::setup(24_000_000 / 1000);
    ::hal::cortex_m3::systick::enable();
}

const MOR_KEY: u8 = 0x37;

fn init_rc_oscillator(freq: RCFreq) {
    let pmc = unsafe { &reg::PMC };

    // Enable MOSCRC
    pmc.mor.update()
        .set_moscrcen(true)
        .set_moscrcf(match freq {
            RCFreq::MHz_4 => reg::pmc::mor::Moscrcf::MHz_4,
            RCFreq::MHz_8 => reg::pmc::mor::Moscrcf::MHz_8,
            RCFreq::MHz_12 => reg::pmc::mor::Moscrcf::MHz_12,
        })
       .set_key(MOR_KEY);
    wait_for!(pmc.st.get().moscrcs());

    pmc.mor.update()
        .set_moscsel(reg::pmc::mor::Moscsel::MOSCRC)
        .set_key(MOR_KEY);
    wait_for!(pmc.st.get().moscsels());
}

fn init_main_oscillator(start_time: u32) {
    let pmc = unsafe { &reg::PMC };

    // Enable MOSCXT
    pmc.mor.update()
        .set_moscxten(true)
        .set_moscrcen(true)
        .set_moscxtst(start_time as u8)
        .set_key(MOR_KEY);
    wait_for!(pmc.st.get().moscxts());

    pmc.mor.update()
        .set_moscsel(reg::pmc::mor::Moscsel::MOSCXT)
        .set_key(MOR_KEY);
    wait_for!(pmc.st.get().moscsels());
}

mod reg {
    use ioreg_proc::ioregs;

    ioregs!(PMC = {
        0x20 => reg32 mor {
            0 => moscxten,
            1 => moscxtby,
            3 => moscrcen,
            4..6 => moscrcf {
                0 => MHz_4,
                1 => MHz_8,
                2 => MHz_12
            },
            8..15 => moscxtst,
            16..23 => key,
            24 => moscsel {
                0 => MOSCRC,
                1 => MOSCXT
            },
            25 => cfden
        },
        0x24 => reg32 mcfr {
            0..15 => mainf: ro,
            16 => mainfrdy: ro
        },
        0x28 => reg32 pllar {
            0..7 => diva,
            8..13 => pllacount,
            16..26 => mula,
            29 => one
        },
        0x30 => reg32 mckr {
            0..1 => css {
                0 => SlowClk,
                1 => MainClk,
                2 => PplaClk,
                3 => UpplClk,
            },
            4..6 => pres {
                0 => CLK,
                1 => CLK_2,
                2 => CLK_4,
                3 => CLK_8,
                4 => CLK_16,
                5 => CLK_32,
                6 => CLK_64,
                7 => CLK_3
            },
            12 => plladiv2,
            13 => uplldiv2
        },
        0x68 => reg32 st {
            0 => moscxts: ro,
            1 => locka: ro,
            3 => mckrdy: ro,
            16 => moscsels: ro,
            17 => moscrcs: ro,
        },
    });

    ioregs!(EEFC = {
        0x0 => reg32 fmr {
            0..11 => fws
        },
        0x8 => reg32 fsr {
            0 => fready: ro
        },
    });

    extern {
        #[link_name="sam3x_iomem_PMC"]
        pub static PMC: pmc::Pmc;
        #[link_name="sam3x_iomem_EEFC0"]
        pub static EEFC0: eefc::Eefc;
        #[link_name="sam3x_iomem_EEFC1"]
        pub static EEFC1: eefc::Eefc;
    }
}
