use rvhwfuzzer_encoding::CsrIndex;

pub mod fcsr;
pub mod misa;
pub mod mstatus;

macro_rules! write_action {
    ($origin:expr, $value:expr, $fn:ident) => {
        $origin.$fn($value)
    };
    ($origin:expr, $value:expr) => {
        return Err(())
    };
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

            pub fn read(&self, id: CsrIndex) -> u32 {
                match id.0 {
                    $( $(
                    $id => self.$field.$read(),
                    )+ )+
                    // @TODO
                    _ => unimplemented!(),
                }
            }

            pub fn write(&mut self, id: CsrIndex, value: u32) -> Result<(), ()> {
                Ok(match id.0 {
                    $( $(
                    $id => write_action!(self.$field, value$(, $write)?),
                    )+ )+
                    // @TODO
                    _ => unimplemented!(),
                })
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct Empty;

impl Empty {
    fn new(ctx: &CsrInitContext) -> Self {
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

csrs! {
    fcsr: fcsr::Fcsr [
        0x001 = (is_available, fflags_read, fflags_write),
        0x002 = (is_available, frm_read, frm_write      ),
        0x003 = (is_available, read, write              ),
    ];

    mstatus: mstatus::MStatus [
        0x300 = (is_available, read, write                  ),
        0x310 = (is_available, mstatush_read, mstatush_write),
    ];
    misa:      misa::MIsa     [ 0x301 = (is_available, read, write) ];

    mvendorid: Empty          [ 0xF11 = (is_available, read, write) ];
    marchid:   Empty          [ 0xF12 = (is_available, read, write) ];
    mimpid:    Empty          [ 0xF13 = (is_available, read, write) ];
    mhartid:   Empty          [ 0xF14 = (is_available, read, write) ];
}

pub struct CsrInitContext {}

pub enum Mode {
    Machine,
    Supervisor,
    User,
}
