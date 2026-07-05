use crate::api::Api;
use crate::bukkit::{
    CommandSender as _, CommandSenderInst, Component, DyeColor, Key, Location, Player, World,
};

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

/// `org.bukkit.World` accessors marshal against a live world.
#[papermc::test]
fn world_accessors(api: &mut Api, world: &World) -> eyre::Result<()> {
    world.environment(api)?;
    let spawn = world.spawn_location(api)?;
    let x = spawn.block_x(api)?;
    let z = spawn.block_z(api)?;
    world.highest_block_y_at(api, x, z)?;
    Ok(())
}

/// MiniMessage parses tags into a Component and escapes untrusted input.
#[papermc::test]
fn mini_message_roundtrip(api: &mut Api) -> eyre::Result<()> {
    Component::mini_message(api, "<green>selftest</green>")?;
    let escaped = crate::bukkit::mini_message::escape_tags(api.jni(), "<red>raw</red>")?;
    eyre::ensure!(
        escaped != "<red>raw</red>",
        "escapeTags left tagged input unchanged"
    );
    Ok(())
}

/// The mirrored `Key.key(String, String)` static factory constructs a Key.
#[papermc::test]
fn key_factory(api: &mut Api) -> eyre::Result<()> {
    Key::key(api, "papermc", "selftest")?;
    Ok(())
}

/// `papermc_enum!` mirrors resolve their Java enum constants.
#[papermc::test]
fn dye_color_variants(api: &mut Api) -> eyre::Result<()> {
    for variant in [DyeColor::White, DyeColor::Orange, DyeColor::Black] {
        let obj = variant.as_java(api.jni())?;
        eyre::ensure!(!obj.is_null(), "{variant:?} resolved to null");
    }
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
