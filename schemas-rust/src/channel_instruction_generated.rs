// automatically generated by the FlatBuffers compiler, do not modify

// @generated

use {
    crate::{execution_request_v1_generated::*, status_v1_generated::*},
    core::{cmp::Ordering, mem},
};

extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};

#[deprecated(
    since = "2.0.0",
    note = "Use associated constants instead. This will no longer be generated in 2021."
)]
pub const ENUM_MIN_CHANNEL_INSTRUCTION_IX_TYPE: u8 = 0;
#[deprecated(
    since = "2.0.0",
    note = "Use associated constants instead. This will no longer be generated in 2021."
)]
pub const ENUM_MAX_CHANNEL_INSTRUCTION_IX_TYPE: u8 = 1;
#[deprecated(
    since = "2.0.0",
    note = "Use associated constants instead. This will no longer be generated in 2021."
)]
#[allow(non_camel_case_types)]
pub const ENUM_VALUES_CHANNEL_INSTRUCTION_IX_TYPE: [ChannelInstructionIxType; 2] = [
    ChannelInstructionIxType::ExecuteV1,
    ChannelInstructionIxType::StatusV1,
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ChannelInstructionIxType(pub u8);
#[allow(non_upper_case_globals)]
impl ChannelInstructionIxType {
    pub const ExecuteV1: Self = Self(0);
    pub const StatusV1: Self = Self(1);

    pub const ENUM_MIN: u8 = 0;
    pub const ENUM_MAX: u8 = 1;
    pub const ENUM_VALUES: &'static [Self] = &[Self::ExecuteV1, Self::StatusV1];
    /// Returns the variant's name or "" if unknown.
    pub fn variant_name(self) -> Option<&'static str> {
        match self {
            Self::ExecuteV1 => Some("ExecuteV1"),
            Self::StatusV1 => Some("StatusV1"),
            _ => None,
        }
    }
}
impl core::fmt::Debug for ChannelInstructionIxType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(name) = self.variant_name() {
            f.write_str(name)
        } else {
            f.write_fmt(format_args!("<UNKNOWN {:?}>", self.0))
        }
    }
}
impl<'a> flatbuffers::Follow<'a> for ChannelInstructionIxType {
    type Inner = Self;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        let b = flatbuffers::read_scalar_at::<u8>(buf, loc);
        Self(b)
    }
}

impl flatbuffers::Push for ChannelInstructionIxType {
    type Output = ChannelInstructionIxType;
    #[inline]
    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {
        flatbuffers::emplace_scalar::<u8>(dst, self.0);
    }
}

impl flatbuffers::EndianScalar for ChannelInstructionIxType {
    type Scalar = u8;
    #[inline]
    fn to_little_endian(self) -> u8 {
        self.0.to_le()
    }
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn from_little_endian(v: u8) -> Self {
        let b = u8::from_le(v);
        Self(b)
    }
}

impl<'a> flatbuffers::Verifiable for ChannelInstructionIxType {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        u8::run_verifier(v, pos)
    }
}

impl flatbuffers::SimpleToVerifyInSlice for ChannelInstructionIxType {}
pub enum ChannelInstructionOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct ChannelInstruction<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ChannelInstruction<'a> {
    type Inner = ChannelInstruction<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table::new(buf, loc),
        }
    }
}

impl<'a> ChannelInstruction<'a> {
    pub const VT_IX_TYPE: flatbuffers::VOffsetT = 4;
    pub const VT_EXECUTE_V1: flatbuffers::VOffsetT = 6;
    pub const VT_STATUS_V1: flatbuffers::VOffsetT = 8;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ChannelInstruction { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ChannelInstructionArgs<'args>,
    ) -> flatbuffers::WIPOffset<ChannelInstruction<'bldr>> {
        let mut builder = ChannelInstructionBuilder::new(_fbb);
        if let Some(x) = args.status_v1 {
            builder.add_status_v1(x);
        }
        if let Some(x) = args.execute_v1 {
            builder.add_execute_v1(x);
        }
        builder.add_ix_type(args.ix_type);
        builder.finish()
    }

    #[inline]
    pub fn ix_type(&self) -> ChannelInstructionIxType {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<ChannelInstructionIxType>(
                    ChannelInstruction::VT_IX_TYPE,
                    Some(ChannelInstructionIxType::ExecuteV1),
                )
                .unwrap()
        }
    }
    #[inline]
    pub fn execute_v1(&self) -> Option<flatbuffers::Vector<'a, u8>> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(
                    ChannelInstruction::VT_EXECUTE_V1,
                    None,
                )
        }
    }
    pub fn execute_v1_nested_flatbuffer(&'a self) -> Option<ExecutionRequestV1<'a>> {
        self.execute_v1().map(|data| {
            use flatbuffers::Follow;
            // Safety:
            // Created from a valid Table for this object
            // Which contains a valid flatbuffer in this slot
            unsafe {
                <flatbuffers::ForwardsUOffset<ExecutionRequestV1<'a>>>::follow(data.bytes(), 0)
            }
        })
    }
    #[inline]
    pub fn status_v1(&self) -> Option<flatbuffers::Vector<'a, u8>> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(
                    ChannelInstruction::VT_STATUS_V1,
                    None,
                )
        }
    }
    pub fn status_v1_nested_flatbuffer(&'a self) -> Option<StatusV1<'a>> {
        self.status_v1().map(|data| {
            use flatbuffers::Follow;
            // Safety:
            // Created from a valid Table for this object
            // Which contains a valid flatbuffer in this slot
            unsafe { <flatbuffers::ForwardsUOffset<StatusV1<'a>>>::follow(data.bytes(), 0) }
        })
    }
}

impl flatbuffers::Verifiable for ChannelInstruction<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        v.visit_table(pos)?
            .visit_field::<ChannelInstructionIxType>("ix_type", Self::VT_IX_TYPE, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>(
                "execute_v1",
                Self::VT_EXECUTE_V1,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>(
                "status_v1",
                Self::VT_STATUS_V1,
                false,
            )?
            .finish();
        Ok(())
    }
}
pub struct ChannelInstructionArgs<'a> {
    pub ix_type: ChannelInstructionIxType,
    pub execute_v1: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub status_v1: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
}
impl<'a> Default for ChannelInstructionArgs<'a> {
    #[inline]
    fn default() -> Self {
        ChannelInstructionArgs {
            ix_type: ChannelInstructionIxType::ExecuteV1,
            execute_v1: None,
            status_v1: None,
        }
    }
}

pub struct ChannelInstructionBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ChannelInstructionBuilder<'a, 'b> {
    #[inline]
    pub fn add_ix_type(&mut self, ix_type: ChannelInstructionIxType) {
        self.fbb_.push_slot::<ChannelInstructionIxType>(
            ChannelInstruction::VT_IX_TYPE,
            ix_type,
            ChannelInstructionIxType::ExecuteV1,
        );
    }
    #[inline]
    pub fn add_execute_v1(
        &mut self,
        execute_v1: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u8>>,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            ChannelInstruction::VT_EXECUTE_V1,
            execute_v1,
        );
    }
    #[inline]
    pub fn add_status_v1(
        &mut self,
        status_v1: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u8>>,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            ChannelInstruction::VT_STATUS_V1,
            status_v1,
        );
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    ) -> ChannelInstructionBuilder<'a, 'b> {
        let start = _fbb.start_table();
        ChannelInstructionBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<ChannelInstruction<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for ChannelInstruction<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("ChannelInstruction");
        ds.field("ix_type", &self.ix_type());
        ds.field("execute_v1", &self.execute_v1());
        ds.field("status_v1", &self.status_v1());
        ds.finish()
    }
}
#[inline]
/// Verifies that a buffer of bytes contains a `ChannelInstruction`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_channel_instruction_unchecked`.
pub fn root_as_channel_instruction(
    buf: &[u8],
) -> Result<ChannelInstruction, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root::<ChannelInstruction>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `ChannelInstruction` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_channel_instruction_unchecked`.
pub fn size_prefixed_root_as_channel_instruction(
    buf: &[u8],
) -> Result<ChannelInstruction, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root::<ChannelInstruction>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `ChannelInstruction` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_channel_instruction_unchecked`.
pub fn root_as_channel_instruction_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<ChannelInstruction<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::root_with_opts::<ChannelInstruction<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `ChannelInstruction` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_channel_instruction_unchecked`.
pub fn size_prefixed_root_as_channel_instruction_with_opts<'b, 'o>(
    opts: &'o flatbuffers::VerifierOptions,
    buf: &'b [u8],
) -> Result<ChannelInstruction<'b>, flatbuffers::InvalidFlatbuffer> {
    flatbuffers::size_prefixed_root_with_opts::<ChannelInstruction<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a ChannelInstruction and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `ChannelInstruction`.
pub unsafe fn root_as_channel_instruction_unchecked(buf: &[u8]) -> ChannelInstruction {
    flatbuffers::root_unchecked::<ChannelInstruction>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed ChannelInstruction and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `ChannelInstruction`.
pub unsafe fn size_prefixed_root_as_channel_instruction_unchecked(
    buf: &[u8],
) -> ChannelInstruction {
    flatbuffers::size_prefixed_root_unchecked::<ChannelInstruction>(buf)
}
#[inline]
pub fn finish_channel_instruction_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<ChannelInstruction<'a>>,
) {
    fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_channel_instruction_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<ChannelInstruction<'a>>,
) {
    fbb.finish_size_prefixed(root, None);
}
