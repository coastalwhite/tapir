use rvhwfuzzer_encoding::CsrIndex;

pub mod fcsr;

macro_rules! csrs {
    (
    $($field:ident: $ty:ty = $id:literal,)+
    $([$subfield_origin:ident] $subfield_read:ident, $subfield_write:ident = $subfield_id:literal),* $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub struct ControlStatusRegisters {
            $(pub $field: $ty,)+
        }

        impl ControlStatusRegisters {
            pub fn new(ctx: &CsrInitContext) -> Self {
                Self {
                    $($field: Csr::new(ctx),)+
                }
            }

            pub fn read(&self, id: CsrIndex) -> u32 {
                match id.0 {
                    $($id => self.$field.read(),)+
                    $($subfield_id => self.$subfield_origin.$subfield_read(),)+
                    // @TODO
                    _ => unimplemented!(),
                }
            }

            pub fn write(&mut self, id: CsrIndex, value: u32) {
                match id.0 {
                    $($id => self.$field.write(value),)+
                    $($subfield_id => self.$subfield_origin.$subfield_write(value),)+
                    // @TODO
                    _ => unimplemented!(),
                }
            }
        }
    };
}

csrs! {
    fcsr: fcsr::Fcsr = 0x003,

    [fcsr] fflags_read, fflags_write = 0x001,
    [fcsr] frm_read, frm_write = 0x002,
}

pub struct CsrInitContext {

}

pub enum Mode {
    Machine,
    Supervisor,
    User,
}

pub trait Csr {
    const MINIMUM_MODE: Mode;

    fn new(ctx: &CsrInitContext) -> Self;

    fn write(&mut self, value: u32);
    fn read(&self) -> u32;
}
