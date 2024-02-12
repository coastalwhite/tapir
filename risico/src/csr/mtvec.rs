use super::CsrInitContext;

#[derive(Debug, Clone, Copy)]
pub struct Mtvec(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MtvecMode {
    Direct = 0b00,
    Vectored = 0b01,
    Reserved10 = 0b10,
    Reserved11 = 0b11,
}

#[repr(u32)]
pub enum TrapCause {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    AtomicStoreAddressMisaligned = 6,
    AtomicStoreAccessFault = 7,
    EcallUmode = 8,
    EcallHSmode = 9,
    EcallVSmode = 10,
    EcallMmode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    AtomicStorePageFault = 15,
}

impl Mtvec {
    #[inline]
    pub fn new(ctx: &CsrInitContext) -> Self {
        Self(0)
    }

    #[inline]
    pub fn read(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn write(&mut self, value: u32) {
        self.0 = value;
    }

    pub fn mode(self) -> MtvecMode {
        match self.0 & 0b11 {
            0b00 => MtvecMode::Direct,
            0b01 => MtvecMode::Vectored,
            0b10 => MtvecMode::Reserved10,
            0b11 => MtvecMode::Reserved11,
            _ => unreachable!(),
        }
    }

    pub fn base(self) -> u32 {
        self.0 & 0xFFFF_FFFC
    }

    pub fn cause_addr(self, trap_cause: TrapCause) -> Option<u32> {
        match self.mode() {
            MtvecMode::Direct => Some(self.base()),
            MtvecMode::Vectored => Some(self.base() + (trap_cause as u32) * 4),
            MtvecMode::Reserved10 => None,
            MtvecMode::Reserved11 => None,
        }
    }

}
