# Nitwittery

A villager-enhancement Minecraft paper server plugin in Rust

# Why Rust?

Because I like it! And I wanted to know if it were possible!

I don't expect this project to become a real plugin that real players use. It's a hobby project to
see what's possible.

# Developer info

See the [ARCHITECTURE.md](ARCHITECTURE.md) for more details on the design and implementation of this
plugin.

Once the required dependencies are installed, the [Makefile](Makefile) is the main entry point for
the build system.

```sh
make
make run
```

## Plugin reloading

If you make Rust changes, you can (sometimes, depending on the change) reload the plugin without
restarting the server.

```sh
make
make run
# ... some time later
make
# and then in game or the server console:
/reload
```

ABI changes, Java changes, or changes to the plugin's `plugin.yml` will require a server restart.

## Dependencies

This project requires Rust and a JDK. It only supports Linux. It probably _could_ support Windows,
but this is a toy project, and I don't have a Windows system available to test with.

### Rust 1.95+

<https://rust-lang.org/tools/install/>

### JDK 25+

```sh
# Fedora
sudo dnf install java-25-openjdk-devel
# Ubuntu 26.04 (use temurin on earlier Ubuntu releases)
sudo apt install openjdk-25-jdk
```

## How to test

If you build with `--features tests` then a `#[papermc::test]` attribute and `/test` command will be
available. This lets you run tests that exercise the Paper API against a running server. Build tests
like:

```rust
#[cfg(feature = "tests")]
mod tests {
    use super::*;

    #[papermc::test]
    fn test_foo(api: &mut Api, world: &World, player: &Player) {
        // ...
    }
}
```

You can run the tests from the server console, or from in-game with the `/test` command. If running
from the console, tests depending on the `player: &Player` fixture will be filtered out since
there's no player available.

![screenshot of the /test command](/docs/tests.png)
