package io.papermc;

import java.lang.ref.Cleaner;
import java.util.concurrent.Callable;

/**
 * Adapts a Rust-side closure (identified by a long id) to {@link Callable}, for
 * use with {@link org.bukkit.scheduler.BukkitScheduler#callSyncMethod}.
 *
 * <p>
 * The closure runs on the main server thread when Bukkit invokes
 * {@link #call()}. The Rust closure is responsible for storing its result in
 * Rust-side state; this bridge always returns null.
 *
 * <p>
 * Each instance registers a {@link Cleaner} action that drops the Rust closure
 * on GC. The cleaner lambda must not capture {@code this}; capturing the
 * primitive id keeps the instance eligible for GC.
 */
public final class RustCallable implements Callable<Object> {
    private static final Cleaner CLEANER = Cleaner.create();

    private final long id;

    public RustCallable(long id) {
        this.id = id;
        final long capturedId = id;
        CLEANER.register(this, () -> bridgeDrop(capturedId));
    }

    @Override
    public Object call() {
        bridgeDispatch(id);
        return null;
    }

    private static native void bridgeDispatch(long id);

    private static native void bridgeDrop(long id);
}
