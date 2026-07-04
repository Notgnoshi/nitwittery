use crate::api::Api;
use crate::bukkit::{CommandSender as _, CommandSenderInst, Location, Player, World};

/// The distributed registry and JNI plumbing work end to end inside the plugin cdylib.
#[papermc::test]
fn class_lookup(api: &mut Api) -> eyre::Result<()> {
    api.class("java/lang/Object")?;
    Ok(())
}

/// Manual-verification helper for runner batching and abort behavior; sleeps on the main thread.
#[papermc::test(ignore = "slow")]
fn slow_smoke(api: &mut Api) -> eyre::Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(250));
    api.class("java/lang/Object")?;
    Ok(())
}

/// Marshalling round-trip through `org.bukkit.Location` accessors against a real world.
#[papermc::test]
fn location_roundtrip(api: &mut Api, world: &World) -> eyre::Result<()> {
    let loc = Location::new(api, world, 1.5, 64.0, -3.25)?;
    eyre::ensure!(loc.x(api)? == 1.5, "x accessor");
    eyre::ensure!(loc.y(api)? == 64.0, "y accessor");
    eyre::ensure!(loc.z(api)? == -3.25, "z accessor");
    eyre::ensure!(loc.world(api)?.is_some(), "world accessor");
    Ok(())
}

/// The invoker fixture is always present and answers `CommandSender#getName()`.
#[papermc::test]
fn sender_name(api: &mut Api, sender: &CommandSenderInst) -> eyre::Result<()> {
    let name = sender.name(api)?;
    eyre::ensure!(!name.is_empty(), "sender name should not be empty");
    Ok(())
}

/// Runs only when a player invoked `/test`; reported as skipped from the console.
#[papermc::test]
fn player_message(api: &mut Api, player: &Player) -> eyre::Result<()> {
    player.send_plain(api, "papermc selftest says hello")?;
    Ok(())
}
