import com.vanniktech.maven.publish.SonatypeHost

plugins {
    id("com.vanniktech.maven.publish") version "0.36.0" apply false
}

subprojects {
    group = "com.oicana"
    version = "0.1.0-alpha.1"

    apply(plugin = "java-library")
    apply(plugin = "com.vanniktech.maven.publish")

    java {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
        withSourcesJar()
        withJavadocJar()
    }

    repositories {
        mavenCentral()
    }

    mavenPublishing {
        publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)
        signAllPublications()

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
