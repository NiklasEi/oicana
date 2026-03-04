plugins {
    `java-library`
    `maven-publish`
    signing
}

group = "com.oicana"
version = "0.1.0-alpha.1"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    withSourcesJar()
    withJavadocJar()
}

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.11.4")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])

            pom {
                name.set("Oicana")
                description.set("Java integration for the Oicana PDF templating engine")
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

    repositories {
        maven {
            name = "OSSRH"
            url = uri("https://s01.oss.sonatype.org/service/local/staging/deploy/maven2/")
            credentials {
                username = findProperty("ossrhUsername") as String? ?: System.getenv("OSSRH_USERNAME")
                password = findProperty("ossrhPassword") as String? ?: System.getenv("OSSRH_PASSWORD")
            }
        }
    }
}

signing {
    isRequired = gradle.taskGraph.hasTask("publishMavenPublicationToOSSRHRepository")
    sign(publishing.publications["maven"])
}
