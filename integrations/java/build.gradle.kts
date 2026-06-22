buildscript {
    repositories {
        mavenCentral()
        gradlePluginPortal()
    }
    dependencies {
        classpath("com.vanniktech:gradle-maven-publish-plugin:0.36.0")
    }
}

subprojects {
    group = "com.oicana"
    version = "0.4.0-rc1"

    apply(plugin = "java-library")
    apply(plugin = "com.vanniktech.maven.publish")

    configure<JavaPluginExtension> {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    repositories {
        mavenCentral()
    }

    afterEvaluate {
        extensions.configure<com.vanniktech.maven.publish.MavenPublishBaseExtension> {
            pom {
                name.set(project.name)
                description.set("Oicana PDF templating engine - ${project.name}")
                url.set("https://oicana.com")
                licenses {
                    license {
                        name.set("PolyForm Noncommercial License 1.0.0")
                        url.set("https://polyformproject.org/licenses/noncommercial/1.0.0/")
                    }
                }
                developers {
                    developer {
                        id.set("oicana")
                        name.set("Oicana")
                        email.set("hello@oicana.com")
                        organization.set("Oicana")
                        organizationUrl.set("https://oicana.com")
                    }
                }
                scm {
                    connection.set("scm:git:git://github.com/oicana/oicana.git")
                    developerConnection.set("scm:git:ssh://github.com:oicana/oicana.git")
                    url.set("https://github.com/oicana/oicana")
                }
            }
        }
    }
}
