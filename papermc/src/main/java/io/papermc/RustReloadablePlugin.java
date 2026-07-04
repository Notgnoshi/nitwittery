package io.papermc;

import java.util.Locale;

import org.bukkit.Bukkit;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.plugin.java.JavaPlugin;

import io.papermc.paper.event.server.ServerResourcesReloadedEvent;

/**
 * Base JavaPlugin that consumer Rust plugins point {@code main:} at in their
 * {@code plugin.yml}. Owns the JNI bootstrap, the Rust tracing-subscriber
 * install, the /reload self-cycle, and the Rust-side enable/disable calls.
 *
 * Plugin authors implementing the Rust {@code papermc::Plugin} trait do not
 * need to subclass this; the same instance serves every Rust plugin that
 * declares this class as its {@code main}.
 */
public class RustReloadablePlugin extends JavaPlugin implements Listener {

    @Override
    public void onEnable() {
        enableRustSide();
    }

    @Override
    public void onDisable() {
        RustPlugin.on_disable();
    }

    /**
     * Stage and load the native libraries, then bring up the Rust side.
     */
    private void enableRustSide() {
        // Normalize `-` to `_` so the key matches what Cargo produces for crate
        // names containing dashes (Cargo rewrites `-` to `_` in the cdylib
        // filename).
        String pluginKey = getName().toLowerCase(Locale.ROOT).replace('-', '_');
        ClassLoader cl = getClass().getClassLoader();
        String loaderPath = NativeLoader.locate("libpapermc_loader.so", "papermc.loader.path",
                pluginKey, cl);
        String stagedPluginPath = NativeLoader.locate("lib" + pluginKey + "_plugin.so",
                "papermc.loader.plugin.path." + pluginKey, pluginKey, cl);
        String pluginPath = NativeLoader.stageVersioned(stagedPluginPath);

        NativeLoader.load(loaderPath);
        RustTracingSubscriber.install(getLogger());
        RustPlugin.on_enable(pluginPath, this);

        getServer().getPluginManager().registerEvents(this, this);
    }

    /**
     * Hook /reload so it cycles the Rust side. Defer one tick so the event-handling
     * stack unwinds before teardown.
     */
    @EventHandler
    public void onResourcesReloaded(ServerResourcesReloadedEvent event) {
        getLogger().info("ServerResourcesReloadedEvent (cause=" + event.getCause()
                + "): cycling the Rust side of " + getName());
        Bukkit.getScheduler().runTaskLater(this, () -> {
            RustPlugin.on_disable();
            enableRustSide();
        }, 1L);
    }
}
