// automatically generated by the FlatBuffers compiler, do not modify


// @generated

use crate::input_type_generated::*;



extern crate flatbuffers;


pub enum ExecutionRequestV1Offset {}
#[derive(Copy, Clone, PartialEq)]

pub struct ExecutionRequestV1<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ExecutionRequestV1<'a> {
  type Inner = ExecutionRequestV1<'a>;
  #[inline]
  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table::new(buf, loc) }
  }
}

impl<'a> ExecutionRequestV1<'a> {
  pub const VT_TIP: flatbuffers::VOffsetT = 4;
  pub const VT_EXECUTION_ID: flatbuffers::VOffsetT = 6;
  pub const VT_IMAGE_ID: flatbuffers::VOffsetT = 8;
  pub const VT_CALLBACK_PROGRAM_ID: flatbuffers::VOffsetT = 10;
  pub const VT_CALLBACK_INSTRUCTION_PREFIX: flatbuffers::VOffsetT = 12;
  pub const VT_FORWARD_OUTPUT: flatbuffers::VOffsetT = 14;
  pub const VT_VERIFY_INPUT_HASH: flatbuffers::VOffsetT = 16;
  pub const VT_INPUT: flatbuffers::VOffsetT = 18;
  pub const VT_INPUT_DIGEST: flatbuffers::VOffsetT = 20;
  pub const VT_MAX_BLOCK_HEIGHT: flatbuffers::VOffsetT = 22;

  #[inline]
  pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    ExecutionRequestV1 { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args ExecutionRequestV1Args<'args>
  ) -> flatbuffers::WIPOffset<ExecutionRequestV1<'bldr>> {
    let mut builder = ExecutionRequestV1Builder::new(_fbb);
    builder.add_max_block_height(args.max_block_height);
    builder.add_tip(args.tip);
    if let Some(x) = args.input_digest { builder.add_input_digest(x); }
    if let Some(x) = args.input { builder.add_input(x); }
    if let Some(x) = args.callback_instruction_prefix { builder.add_callback_instruction_prefix(x); }
    if let Some(x) = args.callback_program_id { builder.add_callback_program_id(x); }
    if let Some(x) = args.image_id { builder.add_image_id(x); }
    if let Some(x) = args.execution_id { builder.add_execution_id(x); }
    builder.add_verify_input_hash(args.verify_input_hash);
    builder.add_forward_output(args.forward_output);
    builder.finish()
  }


  #[inline]
  pub fn tip(&self) -> u64 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<u64>(ExecutionRequestV1::VT_TIP, Some(0)).unwrap()}
  }
  #[inline]
  pub fn execution_id(&self) -> Option<&'a str> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ExecutionRequestV1::VT_EXECUTION_ID, None)}
  }
  #[inline]
  pub fn image_id(&self) -> Option<&'a str> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ExecutionRequestV1::VT_IMAGE_ID, None)}
  }
  #[inline]
  pub fn callback_program_id(&self) -> Option<flatbuffers::Vector<'a, u8>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(ExecutionRequestV1::VT_CALLBACK_PROGRAM_ID, None)}
  }
  #[inline]
  pub fn callback_instruction_prefix(&self) -> Option<flatbuffers::Vector<'a, u8>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(ExecutionRequestV1::VT_CALLBACK_INSTRUCTION_PREFIX, None)}
  }
  #[inline]
  pub fn forward_output(&self) -> bool {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<bool>(ExecutionRequestV1::VT_FORWARD_OUTPUT, Some(false)).unwrap()}
  }
  #[inline]
  pub fn verify_input_hash(&self) -> bool {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<bool>(ExecutionRequestV1::VT_VERIFY_INPUT_HASH, Some(true)).unwrap()}
  }
  #[inline]
  pub fn input(&self) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Input<'a>>>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Input>>>>(ExecutionRequestV1::VT_INPUT, None)}
  }
  #[inline]
  pub fn input_digest(&self) -> Option<flatbuffers::Vector<'a, u8>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(ExecutionRequestV1::VT_INPUT_DIGEST, None)}
  }
  #[inline]
  pub fn max_block_height(&self) -> u64 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<u64>(ExecutionRequestV1::VT_MAX_BLOCK_HEIGHT, Some(0)).unwrap()}
  }
}

impl flatbuffers::Verifiable for ExecutionRequestV1<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    
    v.visit_table(pos)?
     .visit_field::<u64>("tip", Self::VT_TIP, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<&str>>("execution_id", Self::VT_EXECUTION_ID, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<&str>>("image_id", Self::VT_IMAGE_ID, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("callback_program_id", Self::VT_CALLBACK_PROGRAM_ID, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("callback_instruction_prefix", Self::VT_CALLBACK_INSTRUCTION_PREFIX, false)?
     .visit_field::<bool>("forward_output", Self::VT_FORWARD_OUTPUT, false)?
     .visit_field::<bool>("verify_input_hash", Self::VT_VERIFY_INPUT_HASH, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<Input>>>>("input", Self::VT_INPUT, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("input_digest", Self::VT_INPUT_DIGEST, false)?
     .visit_field::<u64>("max_block_height", Self::VT_MAX_BLOCK_HEIGHT, false)?
     .finish();
    Ok(())
  }
}
pub struct ExecutionRequestV1Args<'a> {
    pub tip: u64,
    pub execution_id: Option<flatbuffers::WIPOffset<&'a str>>,
    pub image_id: Option<flatbuffers::WIPOffset<&'a str>>,
    pub callback_program_id: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub callback_instruction_prefix: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub forward_output: bool,
    pub verify_input_hash: bool,
    pub input: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Input<'a>>>>>,
    pub input_digest: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub max_block_height: u64,
}
impl<'a> Default for ExecutionRequestV1Args<'a> {
  #[inline]
  fn default() -> Self {
    ExecutionRequestV1Args {
      tip: 0,
      execution_id: None,
      image_id: None,
      callback_program_id: None,
      callback_instruction_prefix: None,
      forward_output: false,
      verify_input_hash: true,
      input: None,
      input_digest: None,
      max_block_height: 0,
    }
  }
}

pub struct ExecutionRequestV1Builder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ExecutionRequestV1Builder<'a, 'b> {
  #[inline]
  pub fn add_tip(&mut self, tip: u64) {
    self.fbb_.push_slot::<u64>(ExecutionRequestV1::VT_TIP, tip, 0);
  }
  #[inline]
  pub fn add_execution_id(&mut self, execution_id: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_EXECUTION_ID, execution_id);
  }
  #[inline]
  pub fn add_image_id(&mut self, image_id: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_IMAGE_ID, image_id);
  }
  #[inline]
  pub fn add_callback_program_id(&mut self, callback_program_id: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_CALLBACK_PROGRAM_ID, callback_program_id);
  }
  #[inline]
  pub fn add_callback_instruction_prefix(&mut self, callback_instruction_prefix: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_CALLBACK_INSTRUCTION_PREFIX, callback_instruction_prefix);
  }
  #[inline]
  pub fn add_forward_output(&mut self, forward_output: bool) {
    self.fbb_.push_slot::<bool>(ExecutionRequestV1::VT_FORWARD_OUTPUT, forward_output, false);
  }
  #[inline]
  pub fn add_verify_input_hash(&mut self, verify_input_hash: bool) {
    self.fbb_.push_slot::<bool>(ExecutionRequestV1::VT_VERIFY_INPUT_HASH, verify_input_hash, true);
  }
  #[inline]
  pub fn add_input(&mut self, input: flatbuffers::WIPOffset<flatbuffers::Vector<'b , flatbuffers::ForwardsUOffset<Input<'b >>>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_INPUT, input);
  }
  #[inline]
  pub fn add_input_digest(&mut self, input_digest: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ExecutionRequestV1::VT_INPUT_DIGEST, input_digest);
  }
  #[inline]
  pub fn add_max_block_height(&mut self, max_block_height: u64) {
    self.fbb_.push_slot::<u64>(ExecutionRequestV1::VT_MAX_BLOCK_HEIGHT, max_block_height, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ExecutionRequestV1Builder<'a, 'b> {
    let start = _fbb.start_table();
    ExecutionRequestV1Builder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ExecutionRequestV1<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for ExecutionRequestV1<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("ExecutionRequestV1");
      ds.field("tip", &self.tip());
      ds.field("execution_id", &self.execution_id());
      ds.field("image_id", &self.image_id());
      ds.field("callback_program_id", &self.callback_program_id());
      ds.field("callback_instruction_prefix", &self.callback_instruction_prefix());
      ds.field("forward_output", &self.forward_output());
      ds.field("verify_input_hash", &self.verify_input_hash());
      ds.field("input", &self.input());
      ds.field("input_digest", &self.input_digest());
      ds.field("max_block_height", &self.max_block_height());
      ds.finish()
  }
}
#[inline]
/// Verifies that a buffer of bytes contains a `ExecutionRequestV1`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_execution_request_v1_unchecked`.
pub fn root_as_execution_request_v1(buf: &[u8]) -> Result<ExecutionRequestV1, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root::<ExecutionRequestV1>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `ExecutionRequestV1` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_execution_request_v1_unchecked`.
pub fn size_prefixed_root_as_execution_request_v1(buf: &[u8]) -> Result<ExecutionRequestV1, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root::<ExecutionRequestV1>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `ExecutionRequestV1` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_execution_request_v1_unchecked`.
pub fn root_as_execution_request_v1_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<ExecutionRequestV1<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root_with_opts::<ExecutionRequestV1<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `ExecutionRequestV1` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_execution_request_v1_unchecked`.
pub fn size_prefixed_root_as_execution_request_v1_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<ExecutionRequestV1<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root_with_opts::<ExecutionRequestV1<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a ExecutionRequestV1 and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `ExecutionRequestV1`.
pub unsafe fn root_as_execution_request_v1_unchecked(buf: &[u8]) -> ExecutionRequestV1 {
  flatbuffers::root_unchecked::<ExecutionRequestV1>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed ExecutionRequestV1 and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `ExecutionRequestV1`.
pub unsafe fn size_prefixed_root_as_execution_request_v1_unchecked(buf: &[u8]) -> ExecutionRequestV1 {
  flatbuffers::size_prefixed_root_unchecked::<ExecutionRequestV1>(buf)
}
#[inline]
pub fn finish_execution_request_v1_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<ExecutionRequestV1<'a>>) {
  fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_execution_request_v1_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<ExecutionRequestV1<'a>>) {
  fbb.finish_size_prefixed(root, None);
}
