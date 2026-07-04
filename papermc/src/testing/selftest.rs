use crate::api::Api;

/// The distributed registry and JNI plumbing work end to end inside the plugin cdylib.
#[papermc::test]
fn class_lookup(api: &mut Api) -> eyre::Result<()> {
    api.class("java/lang/Object")?;
    Ok(())
}
