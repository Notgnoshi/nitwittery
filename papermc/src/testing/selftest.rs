use crate::api::Api;

/// The distributed registry and JNI plumbing work end to end inside the plugin cdylib.
#[papermc::test]
fn class_lookup(api: &mut Api) -> eyre::Result<()> {
    api.class("java/lang/Object")?;
    Ok(())
}

/// Manual-verification helper for runner batching and abort behavior; sleeps on the main thread.
#[papermc::test(ignore = "slow; run explicitly when exercising the runner")]
fn slow_smoke(api: &mut Api) -> eyre::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(250));
    api.class("java/lang/Object")?;
    Ok(())
}
