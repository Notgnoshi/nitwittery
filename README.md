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

This project requires Rust, a JDK (25+), Gradle (9.5.0), and dprint.

### Rust 1.95+

<https://rust-lang.org/tools/install/>

### JDK 25+

```sh
# Fedora
sudo dnf install java-25-openjdk-devel
# Ubuntu 26.04 (use temurin on earlier Ubuntu releases)
sudo apt install openjdk-25-jdk
```

### Gradle 9.5.0+

We use Gradle 9.5.0 to bootstrap `gradlew`, but you still need a system installation of gradle:

```sh
GRADLE_VERSION=9.5.0
curl -fsSL "https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip" /tmp/gradle.zip
unzip /tmp/gradle.zip -d ~/.local/share/
ln -sf ~/.local/share/gradle-${GRADLE_VERSION}/bin/gradle ~/.local/bin/gradle
```
