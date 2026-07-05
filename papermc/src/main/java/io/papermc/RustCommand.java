package io.papermc;

import java.util.List;

import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;

/**
 * Generic Bukkit Command subclass that forwards execution and tab completion to
 * a Rust handler keyed by handlerId. Registered programmatically via
 * Server.getCommandMap() so plugin.yml never needs to declare commands added by
 * Rust code.
 */
public final class RustCommand extends Command {

    private final long handlerId;

    public RustCommand(String name, long handlerId) {
        super(name);
        this.handlerId = handlerId;
    }

    @Override
    public boolean execute(CommandSender sender, String label, String[] args) {
        return RustPlugin.dispatch_command(handlerId, sender, args);
    }

    /**
     * Forwards to the Rust completer registered under handlerId. A null return from
     * the native side means "no completer" and falls back to Bukkit's default
     * completion (online player names).
     */
    @Override
    public List<String> tabComplete(CommandSender sender, String alias, String[] args) {
        Object result = RustPlugin.dispatch_tab_complete(handlerId, sender, args);
        if (result == null) {
            return super.tabComplete(sender, alias, args);
        }
        @SuppressWarnings("unchecked")
        List<String> completions = (List<String>) result;
        return completions;
    }
}
