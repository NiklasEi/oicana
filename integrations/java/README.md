# Oicana Java Integration

JNI bindings for Oicana, published to Maven Central as `com.oicana:oicana` plus per-platform native JARs.

## Project structure

The API JAR contains the Java classes. Each native subproject packages a single platform-specific shared library as a resource-only JAR. Users add the API JAR and the native JAR(s) for their platform.

## Local development

Prerequisites: JDK 17+, Rust toolchain.

### Build the native library

```bash
cargo build -p oicana_java_native
```

### Copy it to the correct resource directory

```bash
# Example for Linux x86_64:
mkdir -p natives/linux-x86_64/src/main/resources/natives/linux-x86_64
cp ../../target/debug/liboicana_java_native.so natives/linux-x86_64/src/main/resources/natives/linux-x86_64/

# macOS aarch64:
mkdir -p natives/macos-aarch64/src/main/resources/natives/macos-aarch64
cp ../../target/debug/liboicana_java_native.dylib natives/macos-aarch64/src/main/resources/natives/macos-aarch64/
```

### Run tests

```bash
./gradlew :oicana:test
```

### Publish to local Maven repository

```bash
./gradlew publishToMavenLocal
```

This installs all subprojects to `~/.m2/repository/com/oicana/` for testing with example projects.

## Deployment

Publishing is handled by `.github/workflows/publish-integration-java.yml`.

**Trigger:** push a tag `oicana_java-v{version}` or manually dispatch the workflow with a version input.

**Pipeline steps:**
1. Validates that the tag version matches `oicana-java-native/Cargo.toml` and `build.gradle.kts`
2. Builds native libraries for all 5 platforms in parallel
3. Packages and publishes 6 artifacts to Maven Central via the Sonatype Central Portal
4. Creates a GitHub release

**Version bump locations:**
- `integrations/java/build.gradle.kts` (version in `subprojects` block)
- `integrations/java/oicana-java-native/Cargo.toml`

**Required GitHub secrets:**
- `MAVEN_CENTRAL_USERNAME` / `MAVEN_CENTRAL_PASSWORD` (Central Portal user token)
- `GPG_KEY_ID` / `GPG_SIGNING_KEY` / `GPG_PASSPHRASE` (artifact signing)
