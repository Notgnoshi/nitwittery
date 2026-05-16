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
