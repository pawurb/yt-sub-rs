use crate::{store::KvWrapper, users::User};
use eyre::Result;
use wasm_rs_dbg::dbg as wdbg;

pub async fn check_videos(user: User, _kv: &mut impl KvWrapper) -> Result<()> {
    wdbg!("Checking videos for user: {:?}", user);
    Ok(())
}
