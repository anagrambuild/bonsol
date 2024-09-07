use anagram_bonsol_schema::{
    root_as_deploy_v1, root_as_execution_request_v1, root_as_input_set, DeployV1,
    ExecutionRequestV1, InputSet,
};
use anchor_lang::prelude::ProgramError;
use paste::paste;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;
use std::ops::Deref;

macro_rules! impl_anchor_for {
    (
        $type:ident,
        $fn:ident
    ) => {
        paste! {
            #[derive(Clone, Debug)]
            pub struct [<$type Account>]<'a> {
                data: $type<'a>,
                // PhantomData to tie the lifetime to our struct
                _marker: PhantomData<&'a [u8]>,
            }

            impl<'a> anchor_lang::AccountDeserialize for [<$type Account>]<'a> {
                fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                    // SAFETY: We're extending the lifetime of the buffer to 'a.
                    // This is safe as long as the Account doesn't outlive the original buffer.
                    let extended_buf: &'a [u8] = unsafe { std::mem::transmute(*buf) };

                    let root = $fn(extended_buf).map_err(|_| {
                        anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountDidNotDeserialize)
                    })?;

                    Ok(Self {
                        data: root,
                        _marker: PhantomData,
                    })
                }
            }
            impl<'a> anchor_lang::AccountSerialize for [<$type Account>]<'a> {}


            impl anchor_lang::Owner for [<$type Account>]<'_> {
              fn owner() -> Pubkey {
                  crate::ID
              }
            }

            impl<'a> Deref for [<$type Account>]<'a> {
              type Target = $type<'a>;

              fn deref(&self) -> &Self::Target {
                  &self.data
              }
            }

        }
    };
}

// Usage example:
impl_anchor_for!(DeployV1, root_as_deploy_v1);
impl_anchor_for!(ExecutionRequestV1, root_as_execution_request_v1);
impl_anchor_for!(InputSet, root_as_input_set);
