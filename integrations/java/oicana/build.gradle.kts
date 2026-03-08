// Sources and javadoc JARs are configured for all subprojects in the root build file.

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.11.4")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    testRuntimeOnly(project(":oicana-linux-x86_64"))
    testRuntimeOnly(project(":oicana-linux-aarch64"))
    testRuntimeOnly(project(":oicana-macos-x86_64"))
    testRuntimeOnly(project(":oicana-macos-aarch64"))
    testRuntimeOnly(project(":oicana-windows-x86_64"))
}

tasks.test {
    useJUnitPlatform()
}
