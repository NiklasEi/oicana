rootProject.name = "oicana-java"

include("oicana")

val platforms = listOf("linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64", "windows-x86_64")
for (platform in platforms) {
    include("oicana-$platform")
    project(":oicana-$platform").projectDir = file("natives/$platform")
}
