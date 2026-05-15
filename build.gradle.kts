import com.diffplug.gradle.spotless.SpotlessExtension

plugins {
    id("com.diffplug.spotless") version "8.5.1" apply false
}

subprojects {
    layout.buildDirectory.set(rootProject.layout.buildDirectory.dir(name))

    plugins.withId("java") {
        apply(plugin = "com.diffplug.spotless")

        extensions.configure<SpotlessExtension> {
            java {
                target("src/main/java/**/*.java")
                eclipse("4.39").configFile(rootProject.file("eclipse-formatter.xml"))
            }
        }

        tasks.withType<JavaCompile>().configureEach {
            options.compilerArgs.addAll(listOf("-Werror", "-Xlint:all", "-Xlint:-serial"))
        }
    }
}
