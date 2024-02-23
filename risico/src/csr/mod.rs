use rvhwfuzzer_encoding::CsrIndex;

pub mod fcsr;
pub mod mcause;
pub mod mdeleg;
pub mod misa;
pub mod mstatus;
pub mod mtvec;
pub mod counter;

pub enum CsrError {
    UnknownRegister,
    ReadonlyRegister,
}

macro_rules! csrs {
    (
    $( $field:ident: $ty:ty [ $($id:literal = ($is_available:ident, $read:ident$(, $write:ident)? $(,)?) ),+ $(,)? ]; )+
    ) => {
        #[derive(Clone, Debug)]
        pub struct ControlStatusRegisters {
            $(pub $field: $ty,)+
        }

        impl ControlStatusRegisters {
            pub fn new(ctx: &CsrInitContext) -> Self {
                Self {
                    $($field: <$ty>::new(ctx),)+
                }
            }

            pub fn read(&self, id: CsrIndex) -> Result<u32, CsrError> {
                match id.0 {
                    $( $(
                    $id => Ok(self.$field.$read()),
                    )+ )+
                    _ => Err(CsrError::UnknownRegister),
                }
            }

            pub fn write(&mut self, id: CsrIndex, value: u32) -> Result<u32, CsrError> {
                match id.0 {
                    $( $(
                    $id => {
                        #[allow(unused_assignments)]
                        let mut rv = Err(CsrError::ReadonlyRegister);

                        $(
                            rv = Ok(self.$field.$read());
                            self.$field.$write(value);
                        )?

                        rv
                    },
                    )+ )+
                    _ => Err(CsrError::UnknownRegister),
                }
            }

            pub fn update(&mut self, id: CsrIndex, f: impl FnOnce(u32) -> u32) -> Result<u32, CsrError> {
                match id.0 {
                    $( $(
                    $id => {
                        #[allow(unused_assignments)]
                        let mut rv = Err(CsrError::ReadonlyRegister);

                        $(
                            let value = self.$field.$read();
                            rv = Ok(value);
                            self.$field.$write(f(value));
                        )?

                        rv
                    },
                    )+ )+
                    _ => Err(CsrError::UnknownRegister),
                }
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct Empty;

#[derive(Clone, Debug)]
pub struct Simple(u32);

impl Empty {
    fn new(_: &CsrInitContext) -> Self {
        Self
    }

    fn is_available() -> bool {
        true
    }

    fn read(&self) -> u32 {
        0
    }

    fn write(&mut self, _: u32) {}
}

impl Simple {
    fn new(_: &CsrInitContext) -> Self {
        Self(0)
    }

    fn is_available() -> bool {
        true
    }

    pub fn read(&self) -> u32 {
        self.0
    }

    pub fn write(&mut self, value: u32) {
        self.0 = value;
    }
}

csrs! {
    fcsr: fcsr::Fcsr [
        0x001 = (is_available, fflags_read, fflags_write),
        0x002 = (is_available, frm_read, frm_write      ),
        0x003 = (is_available, read, write              ),
    ];

    // Supervisor address translation and protection
    satp:      Empty          [ 0x180 = (is_available, read, write) ];

    mstatus: mstatus::MStatus [
        0x300 = (is_available, read, write                  ),
        0x310 = (is_available, mstatush_read, mstatush_write),
    ];
    misa:      misa::MIsa      [ 0x301 = (is_available, read, write) ];
    medeleg:   mdeleg::MEDeleg [ 0x302 = (is_available, read, write) ];
    mideleg:   mdeleg::MIDeleg [ 0x303 = (is_available, read, write) ];
    mie:       Empty           [ 0x304 = (is_available, read, write) ];
    mtvec:     mtvec::Mtvec    [ 0x305 = (is_available, read, write) ];

    mscratch:  Simple         [ 0x340 = (is_available, read, write) ];
    mepc:      Simple         [ 0x341 = (is_available, read, write) ];
    mcause:    mcause::MCause [ 0x342 = (is_available, read, write) ];

    cycle:     counter::Counter [
        0xC00 = (is_available, low_read, low_write   ),
        0xC80 = (is_available, high_read, high_write ),
    ];
    time:      counter::Counter [
        0xC01 = (is_available, low_read, low_write   ),
        0xC81 = (is_available, high_read, high_write ),
    ];
    instret:   counter::Counter [
        0xC02 = (is_available, low_read, low_write   ),
        0xC82 = (is_available, high_read, high_write ),
    ];

    mvendorid: Empty          [ 0xF11 = (is_available, read, write) ];
    marchid:   Empty          [ 0xF12 = (is_available, read, write) ];
    mimpid:    Empty          [ 0xF13 = (is_available, read, write) ];
    mhartid:   Empty          [ 0xF14 = (is_available, read, write) ];

    // Non-Maskable Interrupts (NMI)
    mnscratch: Empty          [ 0x740 = (is_available, read, write) ];
    mnepc:     Empty          [ 0x741 = (is_available, read, write) ];
    mncause:   Empty          [ 0x742 = (is_available, read, write) ];
    mnstatus:  Empty          [ 0x744 = (is_available, read, write) ];

    // Physical Memory Protection
    pmpcfg0:   Empty          [ 0x3A0 = (is_available, read, write) ];
    pmpcfg1:   Empty          [ 0x3A1 = (is_available, read, write) ];
    pmpcfg2:   Empty          [ 0x3A2 = (is_available, read, write) ];
    pmpcfg3:   Empty          [ 0x3A3 = (is_available, read, write) ];
    pmpcfg4:   Empty          [ 0x3A4 = (is_available, read, write) ];
    pmpcfg5:   Empty          [ 0x3A5 = (is_available, read, write) ];
    pmpcfg6:   Empty          [ 0x3A6 = (is_available, read, write) ];
    pmpcfg7:   Empty          [ 0x3A7 = (is_available, read, write) ];
    pmpcfg8:   Empty          [ 0x3A8 = (is_available, read, write) ];
    pmpcfg9:   Empty          [ 0x3A9 = (is_available, read, write) ];
    pmpcfg10:  Empty          [ 0x3AA = (is_available, read, write) ];
    pmpcfg11:  Empty          [ 0x3AB = (is_available, read, write) ];
    pmpcfg12:  Empty          [ 0x3AC = (is_available, read, write) ];
    pmpcfg13:  Empty          [ 0x3AD = (is_available, read, write) ];
    pmpcfg14:  Empty          [ 0x3AE = (is_available, read, write) ];
    pmpcfg15:  Empty          [ 0x3AF = (is_available, read, write) ];
    pmpaddr0:  Empty          [ 0x3B0 = (is_available, read, write) ];
    pmpaddr1:  Empty          [ 0x3B1 = (is_available, read, write) ];
    pmpaddr2:  Empty          [ 0x3B2 = (is_available, read, write) ];
    pmpaddr3:  Empty          [ 0x3B3 = (is_available, read, write) ];
    pmpaddr4:  Empty          [ 0x3B4 = (is_available, read, write) ];
    pmpaddr5:  Empty          [ 0x3B5 = (is_available, read, write) ];
    pmpaddr6:  Empty          [ 0x3B6 = (is_available, read, write) ];
    pmpaddr7:  Empty          [ 0x3B7 = (is_available, read, write) ];
    pmpaddr8:  Empty          [ 0x3B8 = (is_available, read, write) ];
    pmpaddr9:  Empty          [ 0x3B9 = (is_available, read, write) ];
    pmpaddr10: Empty          [ 0x3BA = (is_available, read, write) ];
    pmpaddr11: Empty          [ 0x3BB = (is_available, read, write) ];
    pmpaddr12: Empty          [ 0x3BC = (is_available, read, write) ];
    pmpaddr13: Empty          [ 0x3BD = (is_available, read, write) ];
    pmpaddr14: Empty          [ 0x3BE = (is_available, read, write) ];
    pmpaddr15: Empty          [ 0x3BF = (is_available, read, write) ];
    pmpaddr16: Empty          [ 0x3C0 = (is_available, read, write) ];
    pmpaddr17: Empty          [ 0x3C1 = (is_available, read, write) ];
    pmpaddr18: Empty          [ 0x3C2 = (is_available, read, write) ];
    pmpaddr19: Empty          [ 0x3C3 = (is_available, read, write) ];
    pmpaddr20: Empty          [ 0x3C4 = (is_available, read, write) ];
    pmpaddr21: Empty          [ 0x3C5 = (is_available, read, write) ];
    pmpaddr22: Empty          [ 0x3C6 = (is_available, read, write) ];
    pmpaddr23: Empty          [ 0x3C7 = (is_available, read, write) ];
    pmpaddr24: Empty          [ 0x3C8 = (is_available, read, write) ];
    pmpaddr25: Empty          [ 0x3C9 = (is_available, read, write) ];
    pmpaddr26: Empty          [ 0x3CA = (is_available, read, write) ];
    pmpaddr27: Empty          [ 0x3CB = (is_available, read, write) ];
    pmpaddr28: Empty          [ 0x3CC = (is_available, read, write) ];
    pmpaddr29: Empty          [ 0x3CD = (is_available, read, write) ];
    pmpaddr30: Empty          [ 0x3CE = (is_available, read, write) ];
    pmpaddr31: Empty          [ 0x3CF = (is_available, read, write) ];
    pmpaddr32: Empty          [ 0x3D0 = (is_available, read, write) ];
    pmpaddr33: Empty          [ 0x3D1 = (is_available, read, write) ];
    pmpaddr34: Empty          [ 0x3D2 = (is_available, read, write) ];
    pmpaddr35: Empty          [ 0x3D3 = (is_available, read, write) ];
    pmpaddr36: Empty          [ 0x3D4 = (is_available, read, write) ];
    pmpaddr37: Empty          [ 0x3D5 = (is_available, read, write) ];
    pmpaddr38: Empty          [ 0x3D6 = (is_available, read, write) ];
    pmpaddr39: Empty          [ 0x3D7 = (is_available, read, write) ];
    pmpaddr40: Empty          [ 0x3D8 = (is_available, read, write) ];
    pmpaddr41: Empty          [ 0x3D9 = (is_available, read, write) ];
    pmpaddr42: Empty          [ 0x3DA = (is_available, read, write) ];
    pmpaddr43: Empty          [ 0x3DB = (is_available, read, write) ];
    pmpaddr44: Empty          [ 0x3DC = (is_available, read, write) ];
    pmpaddr45: Empty          [ 0x3DD = (is_available, read, write) ];
    pmpaddr46: Empty          [ 0x3DE = (is_available, read, write) ];
    pmpaddr47: Empty          [ 0x3DF = (is_available, read, write) ];
    pmpaddr48: Empty          [ 0x3E0 = (is_available, read, write) ];
    pmpaddr49: Empty          [ 0x3E1 = (is_available, read, write) ];
    pmpaddr50: Empty          [ 0x3E2 = (is_available, read, write) ];
    pmpaddr51: Empty          [ 0x3E3 = (is_available, read, write) ];
    pmpaddr52: Empty          [ 0x3E4 = (is_available, read, write) ];
    pmpaddr53: Empty          [ 0x3E5 = (is_available, read, write) ];
    pmpaddr54: Empty          [ 0x3E6 = (is_available, read, write) ];
    pmpaddr55: Empty          [ 0x3E7 = (is_available, read, write) ];
    pmpaddr56: Empty          [ 0x3E8 = (is_available, read, write) ];
    pmpaddr57: Empty          [ 0x3E9 = (is_available, read, write) ];
    pmpaddr58: Empty          [ 0x3EA = (is_available, read, write) ];
    pmpaddr59: Empty          [ 0x3EB = (is_available, read, write) ];
    pmpaddr60: Empty          [ 0x3EC = (is_available, read, write) ];
    pmpaddr61: Empty          [ 0x3ED = (is_available, read, write) ];
    pmpaddr62: Empty          [ 0x3EE = (is_available, read, write) ];
    pmpaddr63: Empty          [ 0x3EF = (is_available, read, write) ];
}

pub struct CsrInitContext {}

#[repr(u8)]
pub enum Mode {
    User = 0b00,
    Supervisor = 0b01,
    Reserved10 = 0b10,
    Machine = 0b11,
}

impl Mode {
    pub const fn take_masked(x: u32) -> Self {
        match x & 0b11 {
            0b00 => Self::User,
            0b01 => Self::Supervisor,
            0b10 => Self::Reserved10,
            0b11 => Self::Machine,
            _ => unreachable!(),
        }
    }
}
